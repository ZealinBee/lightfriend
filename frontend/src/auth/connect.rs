use yew::prelude::*;
use web_sys::{MouseEvent, HtmlInputElement};
use serde_json::json;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsValue;
use crate::config;
use gloo_net::http::Request;
use web_sys::UrlSearchParams;
use web_sys::js_sys::Date;
use crate::connections::whatsapp::WhatsappConnect;
use crate::connections::calendar::CalendarConnect;
use crate::connections::email::EmailConnect;
use crate::connections::tasks::TasksConnect;
use crate::connections::telegram::TelegramConnect;

#[derive(Properties, PartialEq)]
pub struct ConnectProps {
    pub user_id: i32,
    pub sub_tier: Option<String>,
    pub discount: bool,
}
/*
pub struct Connect {
    pub user_id: i32,
}
*/

#[function_component(Connect)]
pub fn connect(props: &ConnectProps) -> Html {
    let error = use_state(|| None::<String>);
    let connecting = use_state(|| false);
    let calendar_connected = use_state(|| false);
    let connecting_tasks = use_state(|| false);
    let all_calendars = use_state(|| false);

    // Predefined providers (you can expand this list)
    let providers = vec![
        ("gmail", "Gmail", "imap.gmail.com", "993"),
        ("privateemail", "PrivateEmail", "mail.privateemail.com", "993"),
        ("outlook", "Outlook", "imap-mail.outlook.com", "993"),
        ("custom", "Custom", "", ""), // Custom option with empty defaults
    ];

    
    // Check token on component mount
    use_effect_with_deps(
        |_| {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    if let Ok(Some(token)) = storage.get_item("token") {
                        web_sys::console::log_1(&format!("Token found in localStorage: {}", token).into());
                    } else {
                        web_sys::console::log_1(&"No token found in localStorage".into());
                    }
                }
            }
            || ()
        },
        (),
    );

    // Clean URL parameters if present (post-callback)
    use_effect_with_deps(
        move |_| {
            if let Some(window) = web_sys::window() {
                if let Ok(search) = window.location().search() {
                    if !search.is_empty() {
                        let params = UrlSearchParams::new_with_str(&search).unwrap();
                        if params.get("code").is_some() || params.get("state").is_some() {
                            web_sys::console::log_1(&"Detected callback parameters, cleaning URL".into());
                            if let Ok(history) = window.history() {
                                let _ = history.push_state_with_url(
                                    &JsValue::NULL,
                                    "",
                                    Some(&window.location().pathname().unwrap_or_default()),
                                );
                            }
                        }
                    }
                }
            }
            || ()
        },
        (),
    );
            html! {
                <div class="connect-section">
                    // Calendar Services
                    <div class="service-group">
                        <h3 class="service-group-title">
                            <i class="fas fa-calendar"></i>
                            {"Calendar Services"}
                        </h3>
                        <div class="service-list">
                            <CalendarConnect 
                                user_id={props.user_id} 
                                sub_tier={props.sub_tier.clone()} 
                                discount={props.discount}
                            />
                        </div>
                    </div>
                    <div class="service-group">
                        <h3 class="service-group-title">
                            <i class="fas fa-calendar"></i>
                            {"Memory Services"}
                        </h3>
                        <div class="service-list">
                     
                            <TasksConnect 
                                user_id={props.user_id}
                                sub_tier={props.sub_tier.clone()}
                                discount={props.discount}
                            />
                        </div>
                    </div>


                    // Email Services
                    <div class="service-group">
                        <h3 class="service-group-title">
                            <i class="fas fa-envelope"></i>
                            {"Email Services"}
                        </h3>
                        <div class="service-list">

                            <EmailConnect 
                                user_id={props.user_id}
                                sub_tier={props.sub_tier.clone()}
                                discount={props.discount}
                            />
                        </div>
                    </div>

                    // Messaging Services 
                    <div class="service-group">
                        <h3 class="service-group-title">
                            <i class="fas fa-comments"></i>
                            {"Messaging Services"}
                        </h3>
                        <div class="service-list">

                            <WhatsappConnect user_id={props.user_id} sub_tier={props.sub_tier.clone()} discount={props.discount}/>
                            {
                                if props.user_id == 1 {
                                    html! {
                                        <TelegramConnect 
                                            user_id={props.user_id} 
                                            sub_tier={props.sub_tier.clone()} 
                                            discount={props.discount}
                                        />
                                    }
                                } else {
                                    html! {
                                        <div class="service-item coming-soon">
                                            <div class="service-header">
                                                <div class="service-name">
                                                    <img src="https://upload.wikimedia.org/wikipedia/commons/8/82/Telegram_logo.svg" alt="Telegram"/>
                                                    {"Telegram"}
                                                    <span class="coming-soon-tag">{"Coming Soon"}</span>
                                                </div>
                                            </div>
                                            <p class="service-description">
                                                {"Send and receive Telegram messages through SMS or voice calls."}
                                            </p>
                                            <button class="connect-button" disabled=true>
                                                {"Connect"}
                                            </button>
                                        </div>
                                    }
                                }
                            }
                        </div>

                        // Outlook Calendar (Coming Soon)
                        <div class="service-item coming-soon">
                            <div class="service-header">
                                <div class="service-name">
                                    <img src="https://upload.wikimedia.org/wikipedia/commons/d/df/Microsoft_Office_Outlook_%282018%E2%80%93present%29.svg" alt="Outlook Calendar"/>
                                    {"Outlook Calendar"}
                                    <span class="coming-soon-tag">{"Coming Soon"}</span>
                                </div>
                            </div>
                            <p class="service-description">
                                {"Manage your Outlook Calendar events through SMS or voice calls."}
                            </p>
                            <button class="connect-button" disabled=true>
                                {"Connect"}
                            </button>
                        </div>
                    </div>

                    if let Some(err) = (*error).as_ref() {
                        <div class="error-message">
                            {err}
                        </div>
                    }
<style>
{".connect-section {
    max-width: 800px;
    margin: 0;
    padding: 0;
    width: 100%;
    box-sizing: border-box;
}

.service-group {
    margin-bottom: 2.5rem;
    background: rgba(30, 30, 30, 0.7);
    border: 1px solid rgba(30, 144, 255, 0.1);
    border-radius: 16px;
    padding: 2rem;
    backdrop-filter: blur(10px);
    width: 100%;
    box-sizing: border-box;
}

@media (max-width: 768px) {
    .service-group {
        padding: 1rem;
        margin-bottom: 1.5rem;
    }
    
    .service-item {
        padding: 1rem;
    }
    
    .service-header {
        flex-direction: column;
        align-items: flex-start;
        gap: 0.5rem;
    }
    
    .service-status-container {
        width: 100%;
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }
    
    .imap-form input,
    .imap-form select {
        width: 100%;
        box-sizing: border-box;
    }
}

.service-group-title {
    font-size: 1.4rem;
    color: #7EB2FF;
    margin-bottom: 1.5rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid rgba(30, 144, 255, 0.1);
}

.service-list {
    display: grid;
    gap: 1.5rem;
    width: 100%;
    box-sizing: border-box;
}

.service-item {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(30, 144, 255, 0.2);
    border-radius: 12px;
    padding: 1.5rem;
    transition: all 0.3s ease;
    width: 100%;
    box-sizing: border-box;
    overflow-wrap: break-word;
    word-wrap: break-word;
    word-break: break-word;
}

.service-item:hover {
    transform: translateY(-2px);
    border-color: rgba(30, 144, 255, 0.4);
    box-shadow: 0 4px 20px rgba(30, 144, 255, 0.1);
}

.service-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1rem;
}

.service-name {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-size: 1.1rem;
    color: #fff;
}

.service-name img {
    width: 24px;
    height: 24px;
}

.service-status {
    font-size: 0.9rem;
    color: #7EB2FF;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.service-description {
    color: #999;
    font-size: 0.95rem;
    line-height: 1.5;
    margin-bottom: 1.5rem;
}

.connect-button, .disconnect-button {
    width: 100%;
    padding: 0.75rem;
    border-radius: 8px;
    font-size: 0.95rem;
    cursor: pointer;
    transition: all 0.3s ease;
    text-align: center;
    border: none;
}

.connect-button {
    background: linear-gradient(45deg, #1E90FF, #4169E1);
    color: white;
}

.connect-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 20px rgba(30, 144, 255, 0.3);
}

.disconnect-button {
    background: transparent;
    border: 1px solid rgba(255, 99, 71, 0.3);
    color: #FF6347;
}

.disconnect-button:hover {
    background: rgba(255, 99, 71, 0.1);
    border-color: rgba(255, 99, 71, 0.5);
}

.imap-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.imap-form input, .imap-form select {
    padding: 0.75rem;
    border-radius: 8px;
    border: 1px solid rgba(30, 144, 255, 0.2);
    background: rgba(0, 0, 0, 0.2);
    color: #fff;
    font-size: 0.95rem;
}

.imap-form input:focus, .imap-form select:focus {
    border-color: rgba(30, 144, 255, 0.4);
    outline: none;
}

.error-message {
    color: #FF6347;
    background: rgba(255, 99, 71, 0.1);
    border: 1px solid rgba(255, 99, 71, 0.2);
    padding: 1rem;
    border-radius: 8px;
    margin-top: 1rem;
    font-size: 0.9rem;
}

.coming-soon {
    opacity: 0.5;
    pointer-events: none;
}

.coming-soon-tag {
    background: rgba(30, 144, 255, 0.1);
    color: #1E90FF;
    font-size: 0.8rem;
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    margin-left: 0.75rem;
}

.test-button {
    background: rgba(76, 175, 80, 0.2);
    color: #4CAF50;
    border: 1px solid rgba(76, 175, 80, 0.3);
    padding: 0.5rem 1rem;
    border-radius: 6px;
    margin-top: 0.75rem;
    cursor: pointer;
    transition: all 0.3s ease;
}

.test-button:hover {
    background: rgba(76, 175, 80, 0.3);
    border-color: rgba(76, 175, 80, 0.4);
}

.calendar-connect-options {
                            display: flex;
                            flex-direction: column;
                            gap: 10px;
                            margin-top: 10px;
                        }
                        .calendar-checkbox {
                            display: flex;
                            align-items: center;
                            gap: 8px;
                            font-size: 14px;
                            color: #666;
                            cursor: pointer;
                        }
                        .calendar-checkbox input[type='checkbox'] {
                            width: 16px;
                            height: 16px;
                            cursor: pointer;
                        }"}
                        {".service-status-container {
                            display: flex;
                            align-items: center;
                            gap: 8px;
                        }
                        .connected-email {
                            font-size: 0.9em;
                            color: #666;
                            font-style: italic;
                        }
                        .gmail-controls {
                            display: flex;
                            gap: 10px;
                            margin-top: 10px;
                        }
                        .test-button {
                            background-color: #4CAF50;
                            color: white;
                            padding: 8px 16px;
                            border: none;
                            border-radius: 4px;
                            cursor: pointer;
                            margin-left: 10px;
                            font-size: 14px;
                        }
                        .test-button:hover {
                            background-color: #45a049;
                        }

                        .service-group {
                            margin-bottom: 2rem;
                        }

                        .service-group:last-child {
                            margin-bottom: 0;
                        }

                        .service-group-title {
                            color: #7EB2FF;
                            font-size: 1.2rem;
                            margin-bottom: 1rem;
                            display: flex;
                            align-items: center;
                            gap: 0.5rem;
                        }

                        .service-group-title i {
                            font-size: 1.1rem;
                        }

                        .service-list {
                            display: grid;
                            gap: 1rem;
                        }

                        .service-item {
                            background: rgba(0, 0, 0, 0.2);
                            border: 1px solid rgba(30, 144, 255, 0.2);
                            border-radius: 8px;
                            padding: 1.5rem;
                            transition: all 0.3s ease;
                        }

                        .service-item:hover {
                            border-color: rgba(30, 144, 255, 0.4);
                            transform: translateY(-2px);
                        }

                        .service-header {
                            display: flex;
                            align-items: center;
                            justify-content: space-between;
                            margin-bottom: 1rem;
                        }

                        .service-name {
                            display: flex;
                            align-items: center;
                            gap: 0.75rem;
                            color: #fff;
                            font-size: 1.1rem;
                        }

                        .service-name img {
                            width: 24px;
                            height: 24px;
                        }

                        .service-status {
                            font-size: 0.9rem;
                            color: #666;
                        }

                        .service-description {
                            color: #999;
                            font-size: 0.9rem;
                            margin-bottom: 1.5rem;
                            line-height: 1.4;
                        }

                        .connect-button {
                            background: linear-gradient(45deg, #1E90FF, #4169E1);
                            color: white;
                            border: none;
                            padding: 0.75rem 1.5rem;
                            border-radius: 6px;
                            font-size: 0.9rem;
                            cursor: pointer;
                            transition: all 0.3s ease;
                            display: inline-flex;
                            align-items: center;
                            gap: 0.5rem;
                            width: 100%;
                            justify-content: center;
                        }

                        .connect-button:hover {
                            transform: translateY(-2px);
                            box-shadow: 0 4px 20px rgba(30, 144, 255, 0.3);
                        }

                        .connect-button.connected {
                            background: rgba(30, 144, 255, 0.1);
                            border: 1px solid rgba(30, 144, 255, 0.3);
                            color: #1E90FF;
                        }

                        .connect-button.connected:hover {
                            background: rgba(30, 144, 255, 0.15);
                        }

                        .disconnect-button {
                            background: transparent;
                            border: 1px solid rgba(255, 99, 71, 0.3);
                            color: #FF6347;
                            padding: 0.75rem 1.5rem;
                            border-radius: 6px;
                            font-size: 0.9rem;
                            cursor: pointer;
                            transition: all 0.3s ease;
                            margin-top: 0.5rem;
                            width: 100%;
                        }

                        .disconnect-button:hover {
                            background: rgba(255, 99, 71, 0.1);
                            border-color: rgba(255, 99, 71, 0.5);
                        }

                        .coming-soon {
                            opacity: 0.5;
                            pointer-events: none;
                        }

                        .coming-soon-tag {
                            background: rgba(30, 144, 255, 0.1);
                            color: #1E90FF;
                            font-size: 0.8rem;
                            padding: 0.25rem 0.5rem;
                            border-radius: 4px;
                            margin-left: 0.5rem;
                        }

                        .error-message {
                            color: #FF6347;
                            font-size: 0.9rem;
                            margin-top: 1rem;
                            padding: 0.75rem;
                            background: rgba(255, 99, 71, 0.1);
                            border-radius: 6px;
                            border: 1px solid rgba(255, 99, 71, 0.2);
                        }

                        @media (max-width: 768px) {
                            .connect-section {
                                padding: 0;
                                margin: 0;
                            }

                            .service-list {
                                grid-template-columns: 1fr;
                            }

                            .service-item {
                                padding: 1rem;
                            }
                        }



                        "}
                    </style>
                </div>
            }

}
