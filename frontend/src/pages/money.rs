use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;
use yew_router::components::Link;
use serde_json::json;
use web_sys::window;
use wasm_bindgen_futures;
use serde_json::Value;
use crate::config;
use gloo_net::http::Request;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
struct UserProfile {
    id: i32,
    email: String,
    sub_tier: Option<String>,
    phone_number: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct PricingProps {
    #[prop_or_default]
    pub user_id: i32,
    #[prop_or_default]
    pub user_email: String,
    #[prop_or_default]
    pub sub_tier: Option<String>,
    #[prop_or_default]
    pub is_logged_in: bool,
    #[prop_or_default]
    pub phone_number: Option<String>,
    #[prop_or_default]
    pub verified: bool,
}

#[derive(Properties, PartialEq, Clone)]
pub struct CheckoutButtonProps {
    pub user_id: i32,
    pub user_email: String,
    pub subscription_type: String,
}

#[function_component(CheckoutButton)]
pub fn checkout_button(props: &CheckoutButtonProps) -> Html {
    let user_id = props.user_id;
    let user_email = props.user_email.clone();
    let subscription_type = props.subscription_type.clone();

    let onclick = {
        let user_id = user_id.clone();
        let subscription_type = subscription_type.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let user_id = user_id.clone();
            let subscription_type = subscription_type.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(token) = window()
                    .and_then(|w| w.local_storage().ok())
                    .flatten()
                    .and_then(|storage| storage.get_item("token").ok())
                    .flatten()
                {
                    let endpoint = if subscription_type == "hard_mode" {
                        format!("{}/api/stripe/hard-mode-subscription-checkout/{}", config::get_backend_url(), user_id)
                    } else {
                        format!("{}/api/stripe/subscription-checkout/{}", config::get_backend_url(), user_id)
                    };

                    let response = Request::post(&endpoint)
                        .header("Authorization", &format!("Bearer {}", token))
                        .send()
                        .await;

                    match response {
                        Ok(resp) => {
                            if let Ok(json) = resp.json::<Value>().await {
                                if let Some(url) = json.get("url").and_then(|u| u.as_str()) {
                                    if let Some(window) = window() {
                                        let _ = window.location().set_href(url);
                                    }
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            });
        })
    };

    let button_text = "Subscribe";

    html! {
        <button class="iq-button signup-button" {onclick}><b>{button_text}</b></button>
    }
}

#[function_component(Pricing)]
pub fn pricing(props: &PricingProps) -> Html {
    let selected_country = use_state(|| "US".to_string());

    let basic_prices: HashMap<String, f64> = HashMap::from([
        ("US".to_string(), 10.00),
        ("FI".to_string(), 25.00),
        ("UK".to_string(), 25.00),
        ("AU".to_string(), 25.00),
    ]);

    let premium_prices: HashMap<String, f64> = HashMap::from([
        ("US".to_string(), 50.00),
        ("FI".to_string(), 70.00),
        ("UK".to_string(), 70.00),
        ("AU".to_string(), 70.00),
    ]);

    let daily_limits: HashMap<String, (i32, i32)> = HashMap::from([
        ("US".to_string(), (10, 15)), // (Basic, Escape) - daily
        ("FI".to_string(), (50, 100)), // (Basic, Escape) - monthly
        ("UK".to_string(), (50, 100)), // (Basic, Escape) - monthly
        ("AU".to_string(), (50, 100)), // (Basic, Escape) - monthly
    ]);

    let overage_rates: HashMap<String, (f64, f64)> = HashMap::from([
        ("US".to_string(), (0.10, 0.20)), // (message cost, voice cost per minute)
        ("FI".to_string(), (0.30, 0.25)),
        ("UK".to_string(), (0.30, 0.25)),
        ("AU".to_string(), (0.40, 0.25)),
    ]);

    let on_country_change = {
        let selected_country = selected_country.clone();
        Callback::from(move |e: Event| {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                selected_country.set(target.value());
            }
        })
    };

    html! {
        <div class="pricing-panel">
            <div class="pricing-header">
                <h1>{"Invest in Your Peace of Mind"}</h1>
                <p>{"Reduce anxiety, sleep better, and live with clarity without the constant pull of your smartphone."}</p>
            </div>

            <div class="country-selector">
                <label for="country">{"Select your country: "}</label>
                <select id="country" onchange={on_country_change}>
                    { for ["US", "FI", "UK", "AU"].iter().map(|&c| html! { <option value={c} selected={*selected_country == c}>{c}</option> }) }
                </select>
            </div>

            <div class="pricing-grid">
                <div class="pricing-card subscription basic">
                    <div class="card-header">
                        <h3>{"Basic Plan"}</h3>
                        <p class="best-for">{"Best for hardcore users who rely only on SMS and calls."}</p>
                        <div class="price">
                            <span class="amount">{format!("€{:.2}", basic_prices.get(&*selected_country).unwrap_or(&0.0))}</span>
                            <span class="period">{"/month"}</span>
                        </div>
                        <div class="includes">
                            <p>{"Subscription includes:"}</p>
                            <ul class="quota-list">
                                <li>{
                                    if *selected_country == "US" {
                                        format!("📞 Up to {} messages or minutes/day", daily_limits.get(&*selected_country).map(|(basic, _)| *basic).unwrap_or(0))
                                    } else {
                                        format!("📞 Up to {} messages or minutes/month", daily_limits.get(&*selected_country).map(|(basic, _)| *basic).unwrap_or(0))
                                    }
                                }</li>
                                <li>{"⚡ Auto-top-up packages (opt-in, credits carry over)"}</li>
                                <li>{"🔍 Includes Internet search & weather"}</li>
                            </ul>
                        </div>
                    </div>
                    {
                        if props.is_logged_in {
                            if !props.verified {
                                html! {
                                    <button class="iq-button disabled" disabled=true>
                                        <b>{"Please verify your account first"}</b>
                                    </button>
                                }
                            } else if props.sub_tier.is_none() {
                                html! {
                                    <CheckoutButton 
                                        user_id={props.user_id} 
                                        user_email={props.user_email.clone()} 
                                        subscription_type="hard_mode"
                                    />
                                }
                            } else if props.sub_tier.as_ref().map_or(false, |tier| tier != "tier 1") {
                                html! {
                                    <CheckoutButton 
                                        user_id={props.user_id} 
                                        user_email={props.user_email.clone()} 
                                        subscription_type="hard_mode"
                                    />
                                }
                            } else {
                                html! {
                                    <button class="iq-button current-plan" disabled=true><b>{"Current Plan"}</b></button>
                                }
                            }
                        } else {
                            let onclick = {
                                Callback::from(move |e: MouseEvent| {
                                    e.prevent_default();
                                    if let Some(window) = web_sys::window() {
                                        if let Ok(Some(storage)) = window.local_storage() {
                                            let _ = storage.set_item("selected_plan", "hard_mode");
                                            let _ = window.location().set_href("/register");
                                        }
                                    }
                                })
                            };
                            html! {
                                <button onclick={onclick} class="iq-button signup-button"><b>{"Get Started"}</b></button>
                            }
                        }
                    }
                </div>

                <div class="pricing-card subscription premium">
                    <div class="premium-tag">{"Save 100+ Hours Monthly*"}</div>
                    <div class="card-header">
                        <h3>{"Escape Plan"}</h3>
                        <p class="best-for">{"Best for users who need access to essentials like messaging apps, email, and calendar."}</p>
                        <div class="price">
                            <span class="amount">{format!("€{:.2}", premium_prices.get(&*selected_country).unwrap_or(&0.0))}</span>
                            <span class="period">{"/month"}</span>
                        </div>
                        <div class="includes">
                            <p>{"Subscription includes:"}</p>
                            <ul class="quota-list">
                                <li>{
                                    if *selected_country == "US" {
                                        format!("📞 Up to {} messages or minutes/day", daily_limits.get(&*selected_country).map(|(_, escape)| *escape).unwrap_or(0))
                                    } else {
                                        format!("📞 Up to {} messages or minutes/month", daily_limits.get(&*selected_country).map(|(_, escape)| *escape).unwrap_or(0))
                                    }
                                }</li>
                                <li>{"⚡ Auto-top-up packages (opt-in, credits carry over)"}</li>
                                <li>{"🎯 Up to 80 filtered notifications/month"}</li>
                                <li>{"🔍 Includes all essentials"}</li>
                            </ul>
                        </div>
                    </div>
                    {
                        if props.is_logged_in {
                            if !props.verified {
                                html! {
                                    <button class="iq-button disabled" disabled=true>
                                        <b>{"Please verify your account first"}</b>
                                    </button>
                                }
                            } else if props.sub_tier.is_none() {
                                html! {
                                    <CheckoutButton 
                                        user_id={props.user_id} 
                                        user_email={props.user_email.clone()} 
                                        subscription_type="world"
                                    />
                                }
                            } else if props.sub_tier.as_ref().map_or(false, |tier| tier != "tier 2") {
                                html! {
                                    <CheckoutButton 
                                        user_id={props.user_id} 
                                        user_email={props.user_email.clone()} 
                                        subscription_type="world"
                                    />
                                }
                            } else {
                                html! {
                                    <button class="iq-button current-plan" disabled=true><b>{"Current Plan"}</b></button>
                                }
                            }
                        } else {
                            let onclick = {
                                Callback::from(move |e: MouseEvent| {
                                    e.prevent_default();
                                    if let Some(window) = web_sys::window() {
                                        if let Ok(Some(storage)) = window.local_storage() {
                                            let _ = storage.set_item("selected_plan", "world");
                                            let _ = window.location().set_href("/register");
                                        }
                                    }
                                })
                            };
                            html! {
                                <button onclick={onclick} class="iq-button signup-button pro-signup"><b>{"Buy Back Your Time"}</b></button>
                            }
                        }
                    }
                </div>
            </div>

            <div class="topup-pricing">
                <h2>{format!("Overage Rates for {}", *selected_country)}</h2>
                <p>{"When you exceed your daily limit, these rates apply. Enable auto-top-up to automatically add credits when you run low. Unused credits carry over indefinitely."}</p>
                <div class="topup-packages">
                    <div class="pricing-card main">
                        <div class="card-header">
                            <div class="package-row">
                                <h3>{"Messages:"}</h3>
                                <div class="price">
                                    <span class="amount">{format!("€{:.2}", overage_rates.get(&*selected_country).map(|(msg, _)| *msg).unwrap_or(0.0))}</span>
                                    <span class="period">{" per message"}</span>
                                </div>
                            </div>
                            <div class="package-row">
                                <h3>{"Voice Calls:"}</h3>
                                <div class="price">
                                    <span class="amount">{format!("€{:.2}", overage_rates.get(&*selected_country).map(|(_, voice)| *voice).unwrap_or(0.0))}</span>
                                    <span class="period">{" per minute"}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                {
                    if props.is_logged_in {
                        html! {
                            <div class="topup-toggle">
                                <p>{"Choose your auto-top-up package size in your account billing."}</p>
                                <button class="iq-button signup-button" onclick={Callback::from(move |e: MouseEvent| {
                                    e.prevent_default();
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href("/billing");
                                    }
                                })}><b>{"Go to Billing"}</b></button>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="topup-toggle">
                                <p>{"Sign up to enable auto-top-up and never run out of messages!"}</p>
                                <button class="iq-button signup-button" onclick={Callback::from(move |e: MouseEvent| {
                                    e.prevent_default();
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href("/register");
                                    }
                                })}><b>{"Sign Up Now"}</b></button>
                            </div>
                        }
                    }
                }
            </div>

            <div class="feature-comparison">
                <h2>{"Feature Comparison"}</h2>
                <table>
                    <thead>
                        <tr>
                            <th>{"Feature"}</th>
                            <th>{"Basic Plan"}</th>
                            <th>{"Escape Plan"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>{"Message/Minute Limit"}</td>
                            <td>{
                                if *selected_country == "US" {
                                    format!("{}/day", daily_limits.get(&*selected_country).map(|(basic, _)| *basic).unwrap_or(0))
                                } else {
                                    format!("{}/month", daily_limits.get(&*selected_country).map(|(basic, _)| *basic).unwrap_or(0))
                                }
                            }</td>
                            <td>{
                                if *selected_country == "US" {
                                    format!("{}/day", daily_limits.get(&*selected_country).map(|(_, escape)| *escape).unwrap_or(0))
                                } else {
                                    format!("{}/month", daily_limits.get(&*selected_country).map(|(_, escape)| *escape).unwrap_or(0))
                                }
                            }</td>
                        </tr>
                        <tr>
                            <td>{"Auto-Top-Up Packages"}</td>
                            <td>{"Available"}</td>
                            <td>{"Available"}</td>
                        </tr>
                        <tr>
                            <td>{"Search Internet with Perplexity"}</td>
                            <td>{"✅"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Fetch Current Weather"}</td>
                            <td>{"✅"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Photo Analysis & Translation (US & AUS only)"}</td>
                            <td>{"✅"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Fetch QR Code data from photo (US & AUS only)"}</td>
                            <td>{"✅"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Fetch & Send WhatsApp Messages"}</td>
                            <td>{"❌"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Email: Fetch + Notifications"}</td>
                            <td>{"❌"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Calendar: Fetch & Create Events"}</td>
                            <td>{"❌"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Tasks: Fetch & Create"}</td>
                            <td>{"❌"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"24/7 Automated Monitoring"}</td>
                            <td>{"❌"}</td>
                            <td>{"✅"}</td>
                        </tr>
                        <tr>
                            <td>{"Filtered Notifications"}</td>
                            <td>{"❌"}</td>
                            <td>{"Up to 80/month"}</td>
                        </tr>
                        <tr>
                            <td>{"Priority Support (24hr Response)"}</td>
                            <td>{"✅"}</td>
                            <td>{"✅ (Enhanced)"}</td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="phone-number-options">
                <div class="phone-number-section">
                    <h2>{"Phone Number Options"}</h2>
                    <div class="options-grid">
                        <div class="option-card">
                            <h3>{"Request New Number"}</h3>
                            <p>{"Need a phone number? We can provide numbers in select countries like US, Finland, UK, and Australia. Due to regulatory restrictions, we cannot provide numbers in many countries including Germany, India, most African countries, and parts of Asia. If your country isn't listed in the pricing above, contact us to check availability."}</p>
                            <a href="mailto:rasmus@ahtava.com?subject=Request New Phone Number">
                                <button class="iq-button signup-button">{"Check Number Availability"}</button>
                            </a>
                        </div>
                        <div class="option-card">
                            <h3>{"Bring Your Own Number"}</h3>
                            <p>{"Use your own Twilio number to get 50% off any plan and enable service in ANY country. Perfect for regions where we can't directly provide numbers (like Germany, India, African countries). This option lets you use our service worldwide while managing your own number through Twilio."}</p>
                            <a href="mailto:rasmus@ahtava.com?subject=Bring Your Own Twilio Number">
                                <button class="iq-button signup-button">{"Contact Us to Set Up"}</button>
                            </a>
                        </div>
                    </div>
                </div>
            </div>

            <div class="pricing-faq">
                <h2>{"Common Questions"}</h2>
                <div class="faq-grid">
                    <details>
                        <summary>{"How does billing work?"}</summary>
                        <p>{"Pricing varies by country. For US customers, Basic Plan includes 10 messages/day and Escape Plan includes 15 messages/day. For all other countries, Basic Plan includes 50 messages/month and Escape Plan includes 100 messages/month. All Escape Plans include 80 filtered notifications/month. Enable auto-top-up (opt-in) to add extra messages/minutes when you reach your limit. Unused credits carry over indefinitely, with caps to control costs. No hidden fees or commitments."}</p>
                    </details>
                    <details>
                        <summary>{"What counts as a message/minute?"}</summary>
                        <p>{"Voice calls are counted by seconds when you call. Messages are counted per query, with free replies if the AI needs clarification. For US customers, limits reset every 24 hours. For all other countries, limits reset monthly."}</p>
                    </details>
                    <details>
                        <summary>{"How does auto-top-up work?"}</summary>
                        <p>{"Enable auto-top-up in your dashboard to automatically add credits when you run low. When enabled, we'll charge your card based on your country's rates. You'll be notified when charges occur, and you can set daily limits to control costs. Unused credits never expire."}</p>
                    </details>
                    <details>
                        <summary>{"How does automatic monitoring work?"}</summary>
                        <p>{"The AI monitors your email/calendar every minute, notifying you of important emails/events based on priority and custom criteria."}</p>
                    </details>
                </div>
            </div>

            <div class="footnotes">
                <p class="footnote">{"* Gen Z spends 4-7 hours daily on phones, often regretting 60% of social media time. "}<a href="https://explodingtopics.com/blog/smartphone-usage-stats" target="_blank" rel="noopener noreferrer">{"Read the study"}</a></p>
                <p class="footnote">{"The dumbphone is sold separately and is not included in any plan."}</p>
            </div>

            <div class="legal-links">
                <Link<Route> to={Route::Terms}>{"Terms & Conditions"}</Link<Route>>
                {" | "}
                <Link<Route> to={Route::Privacy}>{"Privacy Policy"}</Link<Route>>
            </div>

            <style>
                {r#"
                .pricing-panel {
                    position: relative;
                    min-height: 100vh;
                    padding: 6rem 2rem;
                    color: #ffffff;
                    z-index: 1;
                    overflow: hidden;
                }

                .pricing-panel::before {
                    content: '';
                    position: fixed;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100vh;
                    background-image: url('/assets/human_looking_at_field.webp');
                    background-size: cover;
                    background-position: center;
                    background-repeat: no-repeat;
                    opacity: 0.8;
                    z-index: -2;
                    pointer-events: none;
                }

                .pricing-panel::after {
                    content: '';
                    position: fixed;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100vh;
                    background: linear-gradient(
                        to bottom,
                        rgba(26, 26, 26, 0.75) 0%,
                        rgba(26, 26, 26, 0.9) 100%
                    );
                    z-index: -1;
                    pointer-events: none;
                }

                .pricing-header {
                    text-align: center;
                    margin-bottom: 4rem;
                }

                .pricing-header h1 {
                    font-size: 3.5rem;
                    margin-bottom: 1.5rem;
                    background: linear-gradient(45deg, #fff, #7EB2FF);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    font-weight: 700;
                }

                .pricing-header p {
                    color: #999;
                    font-size: 1.2rem;
                    max-width: 600px;
                    margin: 0 auto;
                }

                .country-selector {
                    text-align: center;
                    margin: 2rem 0;
                    background: rgba(30, 30, 30, 0.7);
                    padding: 1.5rem;
                    border-radius: 16px;
                    border: 1px solid rgba(30, 144, 255, 0.15);
                    max-width: 400px;
                    margin: 2rem auto;
                }

                .country-selector label {
                    color: #7EB2FF;
                    margin-right: 1rem;
                    font-size: 1.1rem;
                }

                .country-selector select {
                    padding: 0.8rem;
                    font-size: 1rem;
                    border-radius: 8px;
                    border: 1px solid rgba(30, 144, 255, 0.3);
                    background: rgba(30, 30, 30, 0.9);
                    color: #fff;
                    cursor: pointer;
                    transition: all 0.3s ease;
                }

                .country-selector select:hover {
                    border-color: rgba(30, 144, 255, 0.5);
                }

                .pricing-grid {
                    display: grid;
                    grid-template-columns: repeat(2, 1fr);
                    gap: 2rem;
                    max-width: 1200px;
                    margin: 4rem auto;
                }

                .pricing-card {
                    background: rgba(30, 30, 30, 0.8);
                    border: 1px solid rgba(30, 144, 255, 0.15);
                    border-radius: 24px;
                    padding: 2.5rem;
                    position: relative;
                    transition: transform 0.3s ease, box-shadow 0.3s ease;
                    backdrop-filter: blur(10px);
                }

                .pricing-card:hover {
                    transform: translateY(-5px);
                    box-shadow: 0 8px 32px rgba(30, 144, 255, 0.15);
                    border-color: rgba(30, 144, 255, 0.3);
                }

                .pricing-card.premium {
                    background: rgba(40, 40, 40, 0.85);
                    border: 2px solid rgba(255, 215, 0, 0.3);
                }

                .pricing-card.premium:hover {
                    box-shadow: 0 8px 32px rgba(255, 215, 0, 0.2);
                    border-color: rgba(255, 215, 0, 0.5);
                }

                .popular-tag {
                    position: absolute;
                    top: -12px;
                    right: 24px;
                    background: linear-gradient(45deg, #1E90FF, #4169E1);
                    color: white;
                    padding: 0.5rem 1rem;
                    border-radius: 20px;
                    font-size: 0.9rem;
                    font-weight: 500;
                }

                .premium-tag {
                    position: absolute;
                    top: -12px;
                    left: 24px;
                    background: linear-gradient(45deg, #FFD700, #FFA500);
                    color: white;
                    padding: 0.5rem 1rem;
                    border-radius: 20px;
                    font-size: 0.9rem;
                    font-weight: 500;
                }

                .card-header h3 {
                    color: #7EB2FF;
                    font-size: 2rem;
                    margin-bottom: 1rem;
                }

                .best-for {
                    color: #e0e0e0;
                    font-size: 1.1rem;
                    margin-top: 0.5rem;
                    margin-bottom: 1.5rem;
                    font-style: italic;
                }

                .price {
                    margin: 1.5rem 0;
                    text-align: center;
                }

                .price .amount {
                    font-size: 3rem;
                    color: #fff;
                    font-weight: 700;
                }

                .price .period {
                    color: #999;
                    font-size: 1.1rem;
                }

                .includes {
                    margin-top: 2rem;
                }

                .includes p {
                    color: #7EB2FF;
                    font-size: 1.1rem;
                    margin-bottom: 1rem;
                }

                .quota-list {
                    list-style: none;
                    padding: 0;
                    margin: 0;
                }

                .quota-list li {
                    color: #e0e0e0;
                    padding: 0.5rem 0;
                    font-size: 1.1rem;
                }

                .feature-comparison {
                    max-width: 1000px;
                    margin: 4rem auto;
                    background: rgba(30, 30, 30, 0.8);
                    border: 1px solid rgba(30, 144, 255, 0.15);
                    border-radius: 24px;
                    padding: 2.5rem;
                    backdrop-filter: blur(10px);
                }

                .feature-comparison h2 {
                    color: #7EB2FF;
                    font-size: 2rem;
                    margin-bottom: 2rem;
                    text-align: center;
                }

                .feature-comparison table {
                    width: 100%;
                    border-collapse: collapse;
                    margin-top: 2rem;
                }

                .feature-comparison th, 
                .feature-comparison td {
                    padding: 1rem;
                    text-align: center;
                    border: 1px solid rgba(30, 144, 255, 0.1);
                }

                .feature-comparison th {
                    background: rgba(30, 144, 255, 0.1);
                    color: #7EB2FF;
                    font-weight: 600;
                }

                .feature-comparison td {
                    color: #e0e0e0;
                }

                .topup-pricing {
                    max-width: 1000px;
                    margin: 4rem auto;
                    text-align: center;
                }

                .topup-pricing h2 {
                    color: #7EB2FF;
                    font-size: 2rem;
                    margin-bottom: 1rem;
                }

                .topup-pricing p {
                    color: #999;
                    margin-bottom: 2rem;
                }

                .topup-packages {
                    max-width: 600px;
                    margin: 2rem auto;
                }

                .pricing-card.main {
                    background: rgba(30, 30, 30, 0.8);
                    border: 1px solid rgba(30, 144, 255, 0.15);
                    padding: 2rem;
                }

                .package-row {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 1rem 0;
                    border-bottom: 1px solid rgba(30, 144, 255, 0.15);
                }

                .package-row:last-child {
                    border-bottom: none;
                }

                .package-row h3 {
                    font-size: 1.2rem;
                    margin: 0;
                }

                .package-row .price {
                    margin: 0;
                }

                .package-row .price .amount {
                    font-size: 1.5rem;
                }

                .topup-toggle {
                    margin-top: 2rem;
                    text-align: center;
                }

                .topup-toggle p {
                    color: #999;
                    margin-bottom: 1rem;
                }

                .phone-number-options {
                    max-width: 1200px;
                    margin: 4rem auto;
                }

                .phone-number-section {
                    text-align: center;
                    padding: 2.5rem;
                }

                .phone-number-section h2 {
                    color: #7EB2FF;
                    font-size: 2.5rem;
                    margin-bottom: 2rem;
                }

                .options-grid {
                    display: grid;
                    grid-template-columns: repeat(2, 1fr);
                    gap: 2rem;
                    margin-top: 2rem;
                }

                .option-card {
                    background: rgba(30, 30, 30, 0.8);
                    border: 1px solid rgba(30, 144, 255, 0.15);
                    border-radius: 24px;
                    padding: 2.5rem;
                    backdrop-filter: blur(10px);
                    transition: transform 0.3s ease, box-shadow 0.3s ease;
                }

                .option-card:hover {
                    transform: translateY(-5px);
                    box-shadow: 0 8px 32px rgba(30, 144, 255, 0.15);
                    border-color: rgba(30, 144, 255, 0.3);
                }

                .option-card h3 {
                    color: #7EB2FF;
                    font-size: 1.8rem;
                    margin-bottom: 1rem;
                }

                .option-card p {
                    color: #e0e0e0;
                    margin-bottom: 2rem;
                    font-size: 1.1rem;
                    line-height: 1.6;
                }

                @media (max-width: 968px) {
                    .options-grid {
                        grid-template-columns: 1fr;
                    }
                    
                    .topup-packages {
                        padding: 0 1rem;
                    }
                    
                    .package-row {
                        flex-direction: column;
                        text-align: center;
                        gap: 0.5rem;
                    }
                }

                .pricing-faq {
                    max-width: 800px;
                    margin: 4rem auto;
                }

                .pricing-faq h2 {
                    color: #7EB2FF;
                    font-size: 2rem;
                    margin-bottom: 2rem;
                    text-align: center;
                }

                .faq-grid {
                    display: grid;
                    gap: 1rem;
                }

                .details {
                    background: rgba(30, 30, 30, 0.8);
                    border: 1px solid rgba(30, 144, 255, 0.15);
                    border-radius: 12px;
                    padding: 1.5rem;
                    transition: all 0.3s ease;
                }

                .details:hover {
                    border-color: rgba(30, 144, 255, 0.3);
                }

                summary {
                    color: #7EB2FF;
                    font-size: 1.1rem;
                    cursor: pointer;
                    padding: 0.5rem 0;
                }

                details p {
                    color: #e0e0e0;
                    margin-top: 1rem;
                    line-height: 1.6;
                    padding: 0.5rem 0;
                }

                .footnotes {
                    max-width: 800px;
                    margin: 3rem auto;
                    text-align: center;
                }

                .footnote {
                    color: #999;
                    font-size: 0.9rem;
                }

                .footnote a {
                    color: #7EB2FF;
                    text-decoration: none;
                    transition: color 0.3s ease;
                }

                .footnote a:hover {
                    color: #1E90FF;
                }

                .legal-links {
                    text-align: center;
                    margin-top: 2rem;
                }

                .legal-links a {
                    color: #999;
                    text-decoration: none;
                    transition: color 0.3s ease;
                }

                .legal-links a:hover {
                    color: #7EB2FF;
                }

                .iq-button {
                    background: linear-gradient(45deg, #1E90FF, #4169E1);
                    color: white;
                    border: none;
                    padding: 1rem 2rem;
                    border-radius: 8px;
                    font-size: 1.1rem;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                    width: 100%;
                    margin-top: 2rem;
                }

                .iq-button:hover {
                    transform: translateY(-2px);
                    box-shadow: 0 4px 20px rgba(30, 144, 255, 0.3);
                    background: linear-gradient(45deg, #4169E1, #1E90FF);
                }

                .iq-button.disabled {
                    background: rgba(30, 30, 30, 0.5);
                    cursor: not-allowed;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                }

                .iq-button.disabled:hover {
                    transform: none;
                    box-shadow: none;
                }

                .iq-button.current-plan {
                    background: rgba(30, 144, 255, 0.3);
                    border: 1px solid rgba(30, 144, 255, 0.5);
                    cursor: default;
                }

                .iq-button.current-plan:hover {
                    transform: none;
                    box-shadow: none;
                    background: rgba(30, 144, 255, 0.3);
                }

                @media (max-width: 968px) {
                    .pricing-grid {
                        grid-template-columns: 1fr;
                    }

                    .pricing-header h1 {
                        font-size: 2.5rem;
                    }

                    .pricing-panel {
                        padding: 4rem 1rem;
                    }

                    .feature-comparison {
                        padding: 1.5rem;
                        margin: 2rem auto;
                    }
                }
                "#}
            </style>
        </div>
    }
}
