use quick_xml::Reader;
use quick_xml::events::Event;

use crate::script_steps::parameters::calculation::Calculation;
use crate::utils::attributes::{get_attribute, parse_unescaped_attribute};

const OBFUSCATED: &str = "••••••••";

fn on_off(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

/// Returns the obfuscated placeholder when `obfuscate` is true, or the original
/// calculation when false. Returns `None` if the calculation is absent in both cases.
fn calc_or_obfuscated(calc: Option<String>, obfuscate: bool) -> Option<String> {
    match calc {
        None => None,
        Some(_) if obfuscate => Some(OBFUSCATED.to_string()),
        Some(c) => Some(c),
    }
}

// ── Add Account ───────────────────────────────────────────────────────────────

pub fn sanitize_add_account(step: &str, obfuscate: bool) -> Option<String> {
    let mut name = String::new();
    let mut account_type: Option<String> = None;
    let mut account_name: Option<String> = None;
    let mut password: Option<String> = None;
    let mut privilege_set: Option<String> = None;
    let mut expire_password = false;

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap_or_default(),
                b"List" => {
                    if account_type.is_none() {
                        account_type = get_attribute(&e, "name");
                    }
                }
                b"PrivilegeSetReference" => {
                    privilege_set = parse_unescaped_attribute(&e, "name");
                }
                b"Boolean" => {
                    if get_attribute(&e, "type").as_deref() == Some("Expire password")
                        && get_attribute(&e, "value").as_deref() == Some("True")
                    {
                        expire_password = true;
                    }
                }
                b"Parameter" => match get_attribute(&e, "type").as_deref() {
                    Some("Name") => {
                        account_name = Calculation::from_xml(&mut reader, &e).display();
                    }
                    Some("Password") => {
                        let calc = Calculation::from_xml(&mut reader, &e).display();
                        password = calc_or_obfuscated(calc, obfuscate);
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    if name.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(at) = account_type {
        parts.push(format!("Authenticate via: {at}"));
    }
    if let Some(an) = account_name {
        parts.push(format!("Account Name: {an}"));
    }
    if let Some(pw) = password {
        parts.push(format!("Password: {pw}"));
    }
    if let Some(ps) = privilege_set {
        parts.push(format!("Privilege Set: {ps}"));
    }
    if expire_password {
        parts.push("Expire password".to_string());
    }

    Some(format!("{name} [ {} ]", parts.join(" ; ")))
}

// ── Change Password ───────────────────────────────────────────────────────────

pub fn sanitize_change_password(step: &str, obfuscate: bool) -> Option<String> {
    let mut name = String::new();
    let mut old_password: Option<String> = None;
    let mut new_password: Option<String> = None;
    let mut with_dialog = true;

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap_or_default(),
                b"Boolean" => {
                    if get_attribute(&e, "type").as_deref() == Some("With dialog") {
                        with_dialog = get_attribute(&e, "value").as_deref() == Some("True");
                    }
                }
                b"Parameter" => match get_attribute(&e, "type").as_deref() {
                    Some("Old") => {
                        let calc = Calculation::from_xml(&mut reader, &e).display();
                        old_password = calc_or_obfuscated(calc, obfuscate);
                    }
                    Some("New") => {
                        let calc = Calculation::from_xml(&mut reader, &e).display();
                        new_password = calc_or_obfuscated(calc, obfuscate);
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    if name.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(op) = old_password {
        parts.push(format!("Old Password: {op}"));
    }
    if let Some(np) = new_password {
        parts.push(format!("Password: {np}"));
    }
    parts.push(format!("With dialog: {}", on_off(with_dialog)));

    Some(format!("{name} [ {} ]", parts.join(" ; ")))
}

// ── Delete Account ────────────────────────────────────────────────────────────

pub fn sanitize_delete_account(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut account_name: Option<String> = None;

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap_or_default(),
                b"Parameter" => {
                    if get_attribute(&e, "type").as_deref() == Some("Calculation") {
                        account_name = Calculation::from_xml(&mut reader, &e).display();
                    }
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    if name.is_empty() {
        return None;
    }

    match account_name {
        Some(an) => Some(format!("{name} [ Account Name: {an} ]")),
        None => Some(name),
    }
}

// ── Enable Account ────────────────────────────────────────────────────────────

pub fn sanitize_enable_account(step: &str) -> Option<String> {
    let mut name = String::new();
    let mut account_name: Option<String> = None;
    let mut activate: Option<bool> = None;

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap_or_default(),
                b"Boolean" => {
                    if get_attribute(&e, "type").as_deref() == Some("enable") {
                        activate = Some(get_attribute(&e, "value").as_deref() == Some("True"));
                    }
                }
                b"Parameter" => {
                    if get_attribute(&e, "type").as_deref() == Some("Calculation") {
                        account_name = Calculation::from_xml(&mut reader, &e).display();
                    }
                }
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    if name.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(an) = account_name {
        parts.push(format!("Account Name: {an}"));
    }
    if let Some(act) = activate {
        parts.push(if act {
            "Activate".to_string()
        } else {
            "Deactivate".to_string()
        });
    }

    Some(format!("{name} [ {} ]", parts.join(" ; ")))
}

// ── Re-Login ──────────────────────────────────────────────────────────────────

pub fn sanitize_re_login(step: &str, obfuscate: bool) -> Option<String> {
    let mut name = String::new();
    let mut account_name: Option<String> = None;
    let mut password: Option<String> = None;
    let mut with_dialog = true;

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap_or_default(),
                b"Boolean" => {
                    if get_attribute(&e, "type").as_deref() == Some("With dialog") {
                        with_dialog = get_attribute(&e, "value").as_deref() == Some("True");
                    }
                }
                b"Parameter" => match get_attribute(&e, "type").as_deref() {
                    Some("Name") => {
                        account_name = Calculation::from_xml(&mut reader, &e).display();
                    }
                    Some("Password") => {
                        let calc = Calculation::from_xml(&mut reader, &e).display();
                        password = calc_or_obfuscated(calc, obfuscate);
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    if name.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(an) = account_name {
        parts.push(format!("Account Name: {an}"));
    }
    if let Some(pw) = password {
        parts.push(format!("Password: {pw}"));
    }
    parts.push(format!("With dialog: {}", on_off(with_dialog)));

    Some(format!("{name} [ {} ]", parts.join(" ; ")))
}

// ── Reset Account Password ────────────────────────────────────────────────────

pub fn sanitize_reset_account_password(step: &str, obfuscate: bool) -> Option<String> {
    let mut name = String::new();
    let mut account_name: Option<String> = None;
    let mut password: Option<String> = None;
    let mut expire_password = false;

    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => name = get_attribute(&e, "name").unwrap_or_default(),
                b"Boolean" => {
                    if get_attribute(&e, "type").as_deref() == Some("Password")
                        && get_attribute(&e, "value").as_deref() == Some("True")
                    {
                        expire_password = true;
                    }
                }
                b"Parameter" => match get_attribute(&e, "type").as_deref() {
                    Some("Name") => {
                        account_name = Calculation::from_xml(&mut reader, &e).display();
                    }
                    Some("Password") => {
                        let calc = Calculation::from_xml(&mut reader, &e).display();
                        password = calc_or_obfuscated(calc, obfuscate);
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        buf.clear();
    }

    if name.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    if let Some(an) = account_name {
        parts.push(format!("Account Name: {an}"));
    }
    if let Some(pw) = password {
        parts.push(format!("Password: {pw}"));
    }
    if expire_password {
        parts.push("Expire password".to_string());
    }

    if parts.is_empty() {
        Some(format!("{name} []"))
    } else {
        Some(format!("{name} [ {} ]", parts.join(" ; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Add Account ───────────────────────────────────────────────────────────

    #[test]
    fn test_add_account_minimal() {
        let xml = r#"
            <Step id="134" name="Add Account" enable="True">
                <ParameterValues membercount="3">
                    <Parameter type="AccountType">
                        <List name="FileMaker" value="0"></List>
                    </Parameter>
                    <Parameter type="PrivilegeSetReference">
                        <PrivilegeSetReference id="0" name="&lt;unknown&gt;"></PrivilegeSetReference>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Expire password" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_add_account(xml.trim(), false),
            Some(
                r#"Add Account [ Authenticate via: FileMaker ; Privilege Set: <unknown> ]"#
                    .to_string()
            )
        );
    }

    #[test]
    fn test_add_account_full() {
        let xml = r#"
            <Step id="134" name="Add Account" enable="True">
                <ParameterValues membercount="5">
                    <Parameter type="AccountType">
                        <List name="FileMaker" value="0"></List>
                    </Parameter>
                    <Parameter type="Name">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$name]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="Password">
                        <Calculation datatype="1" position="1">
                            <Calculation>
                                <Text><![CDATA[$pw]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="PrivilegeSetReference">
                        <PrivilegeSetReference id="2" name="[Data Entry Only]"></PrivilegeSetReference>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Expire password" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_add_account(xml.trim(), false),
            Some(r#"Add Account [ Authenticate via: FileMaker ; Account Name: $name ; Password: $pw ; Privilege Set: [Data Entry Only] ; Expire password ]"#.to_string())
        );
    }

    #[test]
    fn test_add_account_obfuscated() {
        let xml = r#"
            <Step id="134" name="Add Account" enable="True">
                <ParameterValues membercount="5">
                    <Parameter type="AccountType">
                        <List name="FileMaker" value="0"></List>
                    </Parameter>
                    <Parameter type="Name">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$name]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="Password">
                        <Calculation datatype="1" position="1">
                            <Calculation>
                                <Text><![CDATA[$pw]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="PrivilegeSetReference">
                        <PrivilegeSetReference id="2" name="[Data Entry Only]"></PrivilegeSetReference>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Expire password" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_add_account(xml.trim(), true),
            Some(r#"Add Account [ Authenticate via: FileMaker ; Account Name: $name ; Password: •••••••• ; Privilege Set: [Data Entry Only] ]"#.to_string())
        );
    }

    // ── Change Password ───────────────────────────────────────────────────────

    #[test]
    fn test_change_password_with_dialog() {
        let xml = r#"
            <Step id="83" name="Change Password" enable="True">
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean type="With dialog" id="128" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_change_password(xml.trim(), false),
            Some("Change Password [ With dialog: On ]".to_string())
        );
    }

    #[test]
    fn test_change_password_no_dialog_obfuscated() {
        let xml = r#"
            <Step id="83" name="Change Password" enable="True">
                <ParameterValues membercount="3">
                    <Parameter type="Old">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$old]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="New">
                        <Calculation datatype="1" position="1">
                            <Calculation>
                                <Text><![CDATA[$new]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="With dialog" id="128" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_change_password(xml.trim(), true),
            Some("Change Password [ Old Password: •••••••• ; Password: •••••••• ; With dialog: Off ]".to_string())
        );
    }

    // ── Delete Account ────────────────────────────────────────────────────────

    #[test]
    fn test_delete_account_with_name() {
        let xml = r#"
            <Step id="135" name="Delete Account" enable="True">
                <ParameterValues membercount="1">
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$account]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_delete_account(xml.trim()),
            Some("Delete Account [ Account Name: $account ]".to_string())
        );
    }

    #[test]
    fn test_delete_account_empty() {
        let xml = r#"<Step id="135" name="Delete Account" enable="True"></Step>"#;
        assert_eq!(
            sanitize_delete_account(xml.trim()),
            Some("Delete Account".to_string())
        );
    }

    // ── Enable Account ────────────────────────────────────────────────────────

    #[test]
    fn test_enable_account_activate() {
        let xml = r#"
            <Step id="137" name="Enable Account" enable="True">
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean value="True" name="Activate" type="enable"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_enable_account(xml.trim()),
            Some("Enable Account [ Activate ]".to_string())
        );
    }

    #[test]
    fn test_enable_account_deactivate_with_name() {
        let xml = r#"
            <Step id="137" name="Enable Account" enable="True">
                <ParameterValues membercount="2">
                    <Parameter type="Calculation">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$account]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean value="False" name="Activate" type="enable"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_enable_account(xml.trim()),
            Some("Enable Account [ Account Name: $account ; Deactivate ]".to_string())
        );
    }

    // ── Re-Login ──────────────────────────────────────────────────────────────

    #[test]
    fn test_re_login_no_dialog() {
        let xml = r#"
            <Step id="138" name="Re-Login" enable="True">
                <ParameterValues membercount="2">
                    <Parameter type="DataSourceReference">
                        <DataSourceReference id="0" name="Current File"></DataSourceReference>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="With dialog" id="128" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_re_login(xml.trim(), false),
            Some("Re-Login [ With dialog: Off ]".to_string())
        );
    }

    #[test]
    fn test_re_login_with_account_and_password_obfuscated() {
        let xml = r#"
            <Step id="138" name="Re-Login" enable="True">
                <ParameterValues membercount="4">
                    <Parameter type="DataSourceReference">
                        <DataSourceReference id="0" name="Current File"></DataSourceReference>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="With dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Name">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$account]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="Password">
                        <Calculation datatype="1" position="1">
                            <Calculation>
                                <Text><![CDATA[$pw]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_re_login(xml.trim(), true),
            Some(
                "Re-Login [ Account Name: $account ; Password: •••••••• ; With dialog: Off ]"
                    .to_string()
            )
        );
    }

    // ── Reset Account Password ────────────────────────────────────────────────

    #[test]
    fn test_reset_account_password_empty() {
        let xml = r#"
            <Step id="136" name="Reset Account Password" enable="True">
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean value="False" type="Password"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_reset_account_password(xml.trim(), false),
            Some("Reset Account Password []".to_string())
        );
    }

    #[test]
    fn test_reset_account_password_full_obfuscated() {
        let xml = r#"
            <Step id="136" name="Reset Account Password" enable="True">
                <ParameterValues membercount="3">
                    <Parameter type="Name">
                        <Calculation datatype="1" position="0">
                            <Calculation>
                                <Text><![CDATA[$account]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="Password">
                        <Calculation datatype="1" position="1">
                            <Calculation>
                                <Text><![CDATA[$pw]]></Text>
                            </Calculation>
                        </Calculation>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean value="True" type="Password"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;
        assert_eq!(
            sanitize_reset_account_password(xml.trim(), true),
            Some("Reset Account Password [ Account Name: $account ; Password: •••••••• ; Expire password ]".to_string())
        );
    }
}
