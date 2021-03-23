import * as vscode from 'vscode';
import { Uri } from 'vscode';
import * as cp from 'child_process';

let templateFileUri: string;
let defaultSolutions: string[];
let solutionFilename: string;
let testcaseFilename: string;
let pollDelay: number;
let intraPollDelay: number;

const INIT_CONTEST_CMD_ID = 'caffeine-for-codeforces.initContest';
const SUBMIT_TO_CONTEST_CMD_ID = 'caffeine-for-codeforces.submitToContest';
const QUIT_CONTEST_CMD_ID = 'caffeine-for-codeforces.quitContest';
const OUTPUT_CHANNEL_NAME = 'Caffeine';
let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;
let caffeineVersion = "caffeine";
let shouldQuit = false;
let currentContestId = -1;

function execShell(cmd: string) {
    return new Promise<string>((resolve, reject) => {
        cp.exec(cmd, (err, out) => {
            if (err) {
                return reject(err);
            }
            return resolve(out ? out : "");
        });
    });
}

function execCaffeine (args: string) {
    return execShell(`caffeine ${args}`).catch(caffeineError);
}

function quitContest() {
    if (currentContestId === -1) {
        notifyError(`You can't quit a contest if you aren't in a contest!`);
        return;
    }
    statusBarItem.backgroundColor =
        new vscode.ThemeColor('statusBarItem.errorBackground');
    shouldQuit = true;
    /* deactivate(); */
}

function log(s: string, newline = true) {
    newline ? outputChannel.appendLine(s) : outputChannel.append(s);
}

function notifyError(e: string) {
    vscode.window.showErrorMessage(e);
}

function caffeineError(err: string) {
    const errmsg = `Failed to execute caffeine (${err}), see
        [github.com/thud/caffeine](https://github.com/thud/caffeine)`;
    notifyError(errmsg);
    log(`${statusBarItem}`);
    statusBarItem.backgroundColor =
        new vscode.ThemeColor('statusBarItem.errorBackground');
    throw new Error(errmsg);
}

function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

type Contest = {
    id: number;
    name: string;
    durationSeconds: number;
    startTimeSeconds: number;
    relativeTimeSeconds: number;
};

async function getContests() {
    try {
        const contestsStr = await execCaffeine("contest list") as string;
        let contests: [string, Contest][] = contestsStr.split("\n-").slice(1)
        .map(
            c => {
                let lines = c.split("\n");
                let name = lines[1].split(" ").slice(3).join(" ");
                return [
                    name,
                    {
                        name,
                        id: parseInt(lines[0].split(" ")[2]),
                        durationSeconds: parseInt(lines[4].split(" ")[3]),
                        startTimeSeconds: parseInt(lines[5].split(" ")[3]),
                        relativeTimeSeconds: parseInt(lines[6].split(" ")[3]),
                    }
                ];
            }
        );
        return new Map(contests);
    } catch (e) {
        notifyError(`Unable to get list of contests ${e}`);
        return new Map([]);
    }
}

async function getUsersToWatch() {
    let users: string[] = [];
    try {
        users.push(
            (await execCaffeine(`user info`) as string)
            .split("\n")[1].split(" ")[2]
        );
    } catch {
        log(`Unable to find default user.`);
    }

    try {
        let friends = JSON.parse(
            await execCaffeine(`user friends -r`) as string
        );
        users = users.concat(friends.result);
    } catch {
        log(`Unable to find user's friends.`);
    }

    return users;
}

async function generateSolutions(pNames: string[]) {
    let templateUri = Uri.file(templateFileUri);
    if (!vscode.workspace.rootPath) {
        const errmsg = `You must be in a folder for the Caffeine extension to
                    function`;
        notifyError(errmsg);
        throw new Error(errmsg);
    }
    let rootUri = Uri.file(vscode.workspace.rootPath);
    log(` -> problem names: ${pNames}`);
    for (let pName of pNames) {
        let uri = Uri.joinPath(
            rootUri,
            solutionFilename.replace('<problem>', pName),
        );
        log(`Copying template to uri ${uri} `, false);
        try {
            await vscode.workspace.fs.copy(
                templateUri,
                uri,
                { overwrite: false },
            );
            log("succeeded");
        } catch (e) {
            log(`failed (${e})`);
        }
    }
}

async function generateTestcasesAndLeftOverSolutions(contestId: number) {
    if (!vscode.workspace.rootPath) {
        const errmsg = `You must be in a folder for the Caffeine extension to
                    function`;
        notifyError(errmsg);
        throw new Error(errmsg);
    }
    log("Generating testcases for the contest...");
    let rootUri = Uri.file(vscode.workspace.rootPath);
    try {
        let testcasesStr = await execCaffeine(
            `contest testcases ${contestId}`
        ) as string;
        let testcases = new Map<string, string[]>();
        let problemIds: string[] = [];
        if (shouldQuit) {
            log(`Quit by user.`);
            return;
        }

        let ps = testcasesStr.split("--- NEW PROBLEM ---\n");
        ps.shift();
        for (let problem of ps) {
            let problemId = problem.split("\n")[0].toLowerCase();
            problemIds.push(problemId);
            log(`Gathering testcases for problem ${problemId}... `, false);

            let ts = problem.split("+++ NEW TESTCASE +++\n");
            ts.shift();
            for (const [i, testcase] of ts.entries()) {
                let fn = testcaseFilename
                    .replace('<problem>',problemId)
                    .replace('<num>', (i>0?i+1:'') as string);
                let uri = Uri.joinPath(
                    rootUri,
                    fn,
                );

                /* check if file already exists. */
                try {
                    await vscode.workspace.fs.stat(uri);
                    log(`(${fn}), `, false);
                } catch {
                    /* try to write file to disk */
                    try {
                        await vscode.workspace.fs.writeFile(
                            uri,
                            Buffer.from(testcase)
                        );
                        log(`${fn}, `, false);
                    } catch (e) {
                        log(`(${fn}) [${e}], `, false);
                    }
                }
            }
            log('DONE');
        }

        await generateSolutions(problemIds);
    } catch (e) {
        log(`Failed to generate testcases: ${e}`);
    }
}

function notifyNewSubmission(user: string, problem: string, verdict: string) {
    vscode.window.showInformationMessage(`NEW SUBMISSION (${user})
                                        Problem: ${problem}
                                        (${verdict})`);
}

function notifyVerdictChange(user: string, problem: string, verdict: string) {
    vscode.window.showInformationMessage(`VERDICT CHANGED (${user})
                                        Problem: ${problem}
                                        (${verdict})`);
}

// TODO get users to watch
async function watchForChanges(contestId: number, startTime: number,
                               duration: number) {
    let submissions = new Map<string, [number, string]>();

    /* TODO get users to watch */
    /* const usersToWatch = ["thud", "SecondThread", "olliep"]; */
    const usersToWatch = await getUsersToWatch();
    log(`Users to watch: ${usersToWatch}`);
    
    /* main loop */
    while (1) {
        if (shouldQuit) {
            log(`Quit by user.`);
            return;
        }
        /* check if contest end time is in the past */
        if (startTime + duration > Date.now()/1000) {
            log(`Contest ended.`);
            /* return; */
        }

        for (const user of usersToWatch) {
            if (shouldQuit) {
                log(`Quit by user.`);
                return;
            }
            try {
                const latestSub = JSON.parse(
                    await execCaffeine(`user status ${user} -rn1`) as string
                ).result[0];

                /* ignore submissions to other contests */
                if (latestSub.contestId != contestId) continue;

                const prevSub = submissions.get(user) || [ 0, '' ];

                if (!submissions.has(user) || latestSub.id != prevSub[0]) {
                    notifyNewSubmission(
                        user,
                        latestSub.problem.index,
                        latestSub.verdict,
                    );
                } else if (latestSub.verdict != prevSub[1]) {
                    notifyVerdictChange(
                        user,
                        latestSub.problem.index,
                        latestSub.verdict,
                    );
                }
                submissions.set(user, [latestSub.id, latestSub.verdict]);
            } catch {
                log(`failed to get user ${user}'s latest submission.`);
            }
            await sleep(intraPollDelay);
        }
        await sleep(pollDelay);
    }
}

async function submitToContest() {
    if (currentContestId === -1) {
        notifyError(`You can't submit to a contest before you have started
                    one!`);
        return;
    }
    if (!vscode.workspace.rootPath) {
        notifyError(`You must be in a folder to submit code.`);
        return;
    }
    try {
        log(`currentContestId: ${currentContestId}`);
        let problemNames: string[] = JSON.parse(await execCaffeine(
            `contest standings ${currentContestId} -r -f1 -n1`
        ) as string).result.problems.map((p: any) => p.index as string);

        const problemId = await vscode.window.showQuickPick(
            problemNames,
            {
                placeHolder: "Problem Index: A, B, C1, ..."
            },
        );
        log(`problemId: ${problemId}`);

        let rootUri = Uri.file(vscode.workspace.rootPath);
        const files = await vscode.workspace.fs.readDirectory(rootUri);
        const filenames = files.map(([filename, _]) => filename);
        const codeFilenames = filenames.filter(
            fn => fn.includes(".") && !fn.endsWith(".txt")
        );
        const chosenFilename = Uri.joinPath(
            Uri.file(vscode.workspace.rootPath),
            await vscode.window.showQuickPick(
                codeFilenames,
                {
                    placeHolder: "Code Filename: a.cpp, b.cpp, ...",
                },
            ) as string,
        );
        log(`chosenFilename: ${chosenFilename.path}`);
        log(`caffeine submit ${currentContestId} ${problemId} ${chosenFilename.path}`);

        log(await execCaffeine(
            `submit ${currentContestId} ${problemId} ${chosenFilename.path}`
        ) as string);
    } catch (e) {
        notifyError(`Failed to submit to problem: ${e}`);
    }
}

function getFromConfig() {
    const config = vscode.workspace.getConfiguration();
    templateFileUri = config.get('caffeine-for-codeforces.filenames.templateFileLocation') as string;
    defaultSolutions = config.get('caffeine-for-codeforces.filenames.defaultSolutionNames') as string[];
    solutionFilename = config.get('caffeine-for-codeforces.filenames.solutionFilename') as string;
    testcaseFilename = config.get('caffeine-for-codeforces.filenames.testcaseFilename') as string;
    pollDelay = 1000*(config.get('caffeine-for-codeforces.rateLimiting.pollDelay') as number);
    intraPollDelay = 1000*(config.get('caffeine-for-codeforces.rateLimiting.intraPollDelay') as number);
}

async function initContest() {
    shouldQuit = false;
    log('Initialising new Codeforces contest...');

    await execCaffeine("--version")
        .then((v) => { caffeineVersion = v || "caffeine" })
        .catch(caffeineError);

    if (shouldQuit) {
        log(`Quit by user.`);
        return;
    }

    getFromConfig();

    const contests = await getContests();

    /* get contest details with vscode's QuickPick. */
    let names: string[] = [];
    for (const k of contests.keys()) { names.push(k as string) }
    const contestFullName = await vscode.window.showQuickPick(
        names,
        {
            placeHolder: "Global Round 10, Educational Round 8, ..."
        },
    );
    if (!contestFullName) { throw new Error("Failed to select a contest."); }

    if (shouldQuit) {
        log(`Quit by user.`);
        return;
    }

    statusBarItem.text = `$(testing-cancel-icon) ${contestFullName}`;
    const contest = contests.get(contestFullName) as Contest;
    if (!contest) { throw new Error("Failed to select a contest."); }
    log(`Selected contest ${contestFullName} (${contest.id})`);
    currentContestId = contest.id;

    /* Fix any time skew */
    contest.startTimeSeconds =
        Math.floor(Date.now()/1000) + contest.relativeTimeSeconds;

    if (contest.relativeTimeSeconds < 0) {
        /* generate default solutions if set. */
        if (defaultSolutions) { await generateSolutions(defaultSolutions); }

        /* sleep until the start of the contest. */
        const sleeptime = (10-contest.relativeTimeSeconds)*1000;
        log(`Sleeping ${sleeptime} to the start.`);
        await sleep(Math.max(0, sleeptime));
    } else {
        log(`Contest has already started.`);
    }

    /* now inside contest time (or after it has finished) */

    /* download testcases and put into files */
    await generateTestcasesAndLeftOverSolutions(contest.id);

    await watchForChanges(
        contest.id,
        contest.startTimeSeconds,
        contest.durationSeconds,
    );

    log(`Done.`);
    if (shouldQuit) {
        statusBarItem.hide();
    }
}

export async function activate(context: vscode.ExtensionContext) {
    shouldQuit = false;
    outputChannel = vscode.window.createOutputChannel(OUTPUT_CHANNEL_NAME);

	let initContestCmd = vscode.commands.registerCommand(
        INIT_CONTEST_CMD_ID,
        initContest,
    );
	let submitToContestCmd = vscode.commands.registerCommand(
        SUBMIT_TO_CONTEST_CMD_ID,
        submitToContest,
    );
	let quitContestCmd = vscode.commands.registerCommand(
        QUIT_CONTEST_CMD_ID,
        quitContest,
    );

    statusBarItem = vscode.window.createStatusBarItem(
        vscode.StatusBarAlignment.Right,
        10000,
    );
    statusBarItem.command = QUIT_CONTEST_CMD_ID;
    statusBarItem.text = caffeineVersion;
	context.subscriptions.push(statusBarItem);
    /* statusBarItem.backgroundColor = "statusBarItem.activeBackground"; */
    statusBarItem.show();

	context.subscriptions.push(initContestCmd);
	context.subscriptions.push(submitToContestCmd);
	context.subscriptions.push(quitContestCmd);
}

export function deactivate() {
    shouldQuit = false;
    currentContestId = -1;
}
