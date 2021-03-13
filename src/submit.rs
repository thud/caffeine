use headless_chrome::{util::Wait, Browser, Tab};
use std::io::prelude::*;
use std::io::stdin as ioStdin;

#[cfg(feature = "debug-screenshot")]
use headless_chrome::protocol::page::ScreenshotFormat;
#[cfg(feature = "debug-screenshot")]
use std::fs::File;

const TIMEOUT_DELAY: u64 = 10;

pub fn submit_from_string(
    src: &str,
    contest_id: i64,
    problem_index: &str,
    handle: &str,
    password: &str,
    program_type_id: i64,
    mirror: Option<u8>,
) -> Result<(), String> {
    let browser = match Browser::default() {
        Ok(b) => b,
        Err(e) => return Err(format!("headless_chrome: {}", e)),
    };

    let tab = match browser.wait_for_initial_tab() {
        Ok(t) => t,
        Err(e) => return Err(format!("headless_chrome: {}", e)),
    };
    tab.set_default_timeout(std::time::Duration::from_secs(TIMEOUT_DELAY));

    let stub = match mirror {
        Some(1) => "https://m1.codeforces.com/",
        Some(2) => "https://m2.codeforces.com/",
        Some(3) => "https://m3.codeforces.com/",
        _ => "https://codeforces.com/",
    }
    .to_owned();
    let enterurl = stub.to_string() + r"enter";
    let submiturl =
        stub.to_string() + r"contest/" + &contest_id.to_string() + r"/submit";
    let mysubsurl =
        stub.to_string() + r"contest/" + &contest_id.to_string() + r"/my";

    eprintln!("attempting login");
    let res = attempt_tab_login(&tab, &enterurl, handle, password);
    if let Err(e) = res {
        return Err(format!("headless_chrome: {}", e));
    }
    let logged_in =
        Wait::with_timeout(std::time::Duration::from_secs(TIMEOUT_DELAY))
            .until(|| {
                if tab.get_url() == stub {
                    Some(true)
                } else {
                    match tab.find_element("span.error") {
                        Ok(_) => Some(false),
                        Err(_) => None,
                    }
                }
            });

    match logged_in {
        Ok(true) => {
            eprintln!("login successful");
            let res = attempt_tab_submit(
                &tab,
                &submiturl,
                src,
                problem_index,
                program_type_id,
            );
            if let Err(e) = res {
                return Err(format!("headless_chrome: {}", e));
            }
            let mut i = 0;
            let successful = Wait::with_timeout(
                std::time::Duration::from_secs(TIMEOUT_DELAY),
            )
            .until(|| {
                if tab.get_url() == mysubsurl {
                    Some(true)
                } else {
                    match tab.find_elements(".error") {
                        Ok(v) => {
                            // mirrors don't show an invisible span.error
                            // therefore they need separate handling.
                            match mirror {
                                Some(1..=3) => Some(false),
                                _ => {
                                    if v.len() > 1 {
                                        Some(false)
                                    } else {
                                        None
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            #[cfg(feature = "debug-screenshot")]
                            let _ = debug_screenshot(
                                &tab, &format!("ss_waitingforres/{}.jpg", i)
                            );

                            i += 1;
                            None
                        }
                    }
                }
            });
            
            #[cfg(feature = "debug-screenshot")]
            let _ = debug_screenshot(&tab, "ss_aftersubmission.jpg");

            match successful {
                Ok(true) => Ok(()),
                Ok(false) => {
                    Err("failed to submit problem, error on submissions page"
                        .to_string())
                }
                Err(e) => Err(format!("headless_chrome: {}", e)),
            }
        }
        Ok(false) => {
            Err("login unsuccessful, incorrect username or password"
                .to_string())
        }
        Err(e) => Err(format!("headless_chrome: {}", e)),
    }
}

pub fn grab_text_from_stdin() -> Option<String> {
    if atty::isnt(atty::Stream::Stdin) {
        let mut stdin = ioStdin();
        let mut res = String::new();
        let mut line = String::new();
        while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
            if n_bytes == 0 {
                break;
            }
            res += &line;
            line.clear();
        }
        Some(res)
    } else {
        None
    }
}

fn attempt_tab_login(
    tab: &Tab,
    url: &str,
    username: &str,
    password: &str,
) -> Result<(), failure::Error> {
    tab.navigate_to(url)?.wait_until_navigated()?;
    tab.evaluate(
        &format!(
            "$('#handleOrEmail').val('{}');
            $('#password').val('{}');
            $('input#handleOrEmail').closest('form').submit();",
            username, password
        ),
        true,
    )?;
    Ok(())
}

fn attempt_tab_submit(
    tab: &Tab,
    url: &str,
    src: &str,
    problem_index: &str,
    program_type_id: i64,
) -> Result<(), failure::Error> {
    tab.navigate_to(url)?.wait_until_navigated()?;
    tab.evaluate(
        &format!(
            "$('[name=problemIndex],[name=submittedProblemIndex]').val('{}');
            $('[name=programTypeId]').val('{}');",
            problem_index, program_type_id
        ),
        true,
    )?;
    if let Ok(e) = tab.wait_for_element("#toggleEditorCheckbox") {
        tab.evaluate(
            "$('#toggleEditorCheckbox').prop('checked',false);",
            true,
        )?;
        e.click()?;
    }

    #[cfg(feature = "debug-screenshot")]
    let _ = debug_screenshot(tab, "ss_beforesubmit.jpg")?;
    
    tab.evaluate(&format!("$('[name=source]').val({:?});", src), true)?;
    tab.wait_for_element("input[value=Submit]")?.click()?;
    Ok(())
}

#[cfg(feature = "debug-screenshot")]
fn debug_screenshot(tab: &Tab, filename: &str) -> Result<(), failure::Error> {
    let _jpeg_data =
        tab.capture_screenshot(ScreenshotFormat::JPEG(Some(75)), None, true)?;
    let mut f = File::create(filename)?;
    f.write_all(&_jpeg_data)?;
    Ok(())
}
