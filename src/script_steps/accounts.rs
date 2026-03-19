use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

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

fn set_step_name(name: &mut String, event: &BytesStart<'_>) {
    if event.name().as_ref() == b"Step" {
        *name = get_attribute(event, "name").unwrap_or_default();
    }
}

fn attribute_matches(event: &BytesStart<'_>, key: &str, expected: &str) -> bool {
    get_attribute(event, key).as_deref() == Some(expected)
}

fn parse_calculation(reader: &mut Reader<&[u8]>, event: &BytesStart<'_>) -> Option<String> {
    Calculation::from_xml(reader, event).display()
}

fn parse_secret(
    reader: &mut Reader<&[u8]>,
    event: &BytesStart<'_>,
    obfuscate: bool,
) -> Option<String> {
    calc_or_obfuscated(parse_calculation(reader, event), obfuscate)
}

fn parse_step<T, F>(step: &str, mut state: T, mut handle_event: F) -> T
where
    F: FnMut(&mut Reader<&[u8]>, &BytesStart<'_>, &mut T),
{
    let mut reader = Reader::from_str(step);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(event)) => handle_event(&mut reader, &event, &mut state),
            _ => {}
        }

        buf.clear();
    }

    state
}

fn push_labeled_part(parts: &mut Vec<String>, label: &str, value: Option<String>) {
    if let Some(value) = value {
        parts.push(format!("{label}: {value}"));
    }
}

fn push_flag(parts: &mut Vec<String>, enabled: bool, label: &str) {
    if enabled {
        parts.push(label.to_string());
    }
}

fn format_step(name: String, parts: Vec<String>) -> Option<String> {
    if name.is_empty() {
        return None;
    }

    if parts.is_empty() {
        Some(name)
    } else {
        Some(format!("{name} [ {} ]", parts.join(" ; ")))
    }
}

fn format_step_with_empty_brackets(name: String, parts: Vec<String>) -> Option<String> {
    if name.is_empty() {
        return None;
    }

    if parts.is_empty() {
        Some(format!("{name} []"))
    } else {
        Some(format!("{name} [ {} ]", parts.join(" ; ")))
    }
}

struct AddAccountState {
    name: String,
    account_type: Option<String>,
    account_name: Option<String>,
    password: Option<String>,
    privilege_set: Option<String>,
    expire_password: bool,
}

struct ChangePasswordState {
    name: String,
    old_password: Option<String>,
    new_password: Option<String>,
    with_dialog: bool,
}

struct DeleteAccountState {
    name: String,
    account_name: Option<String>,
}

struct EnableAccountState {
    name: String,
    account_name: Option<String>,
    activate: Option<bool>,
}

struct ReLoginState {
    name: String,
    account_name: Option<String>,
    password: Option<String>,
    with_dialog: bool,
}

struct ResetAccountPasswordState {
    name: String,
    account_name: Option<String>,
    password: Option<String>,
    expire_password: bool,
}

pub fn sanitize_add_account(step: &str, obfuscate: bool) -> Option<String> {
    let state = parse_step(
        step,
        AddAccountState {
            name: String::new(),
            account_type: None,
            account_name: None,
            password: None,
            privilege_set: None,
            expire_password: false,
        },
        |reader, event, state| match event.name().as_ref() {
            b"Step" => set_step_name(&mut state.name, event),
            b"List" => {
                if state.account_type.is_none() {
                    state.account_type = get_attribute(event, "name");
                }
            }
            b"PrivilegeSetReference" => {
                state.privilege_set = parse_unescaped_attribute(event, "name");
            }
            b"Boolean" => {
                if attribute_matches(event, "type", "Expire password")
                    && attribute_matches(event, "value", "True")
                {
                    state.expire_password = true;
                }
            }
            b"Parameter" => match get_attribute(event, "type").as_deref() {
                Some("Name") => state.account_name = parse_calculation(reader, event),
                Some("Password") => state.password = parse_secret(reader, event, obfuscate),
                _ => {}
            },
            _ => {}
        },
    );

    let mut parts: Vec<String> = Vec::new();
    push_labeled_part(&mut parts, "Authenticate via", state.account_type);
    push_labeled_part(&mut parts, "Account Name", state.account_name);
    push_labeled_part(&mut parts, "Password", state.password);
    push_labeled_part(&mut parts, "Privilege Set", state.privilege_set);
    push_flag(&mut parts, state.expire_password, "Expire password");

    format_step(state.name, parts)
}

pub fn sanitize_change_password(step: &str, obfuscate: bool) -> Option<String> {
    let state = parse_step(
        step,
        ChangePasswordState {
            name: String::new(),
            old_password: None,
            new_password: None,
            with_dialog: true,
        },
        |reader, event, state| match event.name().as_ref() {
            b"Step" => set_step_name(&mut state.name, event),
            b"Boolean" => {
                if attribute_matches(event, "type", "With dialog") {
                    state.with_dialog = attribute_matches(event, "value", "True");
                }
            }
            b"Parameter" => match get_attribute(event, "type").as_deref() {
                Some("Old") => state.old_password = parse_secret(reader, event, obfuscate),
                Some("New") => state.new_password = parse_secret(reader, event, obfuscate),
                _ => {}
            },
            _ => {}
        },
    );

    let mut parts: Vec<String> = Vec::new();
    push_labeled_part(&mut parts, "Old Password", state.old_password);
    push_labeled_part(&mut parts, "Password", state.new_password);
    parts.push(format!("With dialog: {}", on_off(state.with_dialog)));

    format_step(state.name, parts)
}

pub fn sanitize_delete_account(step: &str) -> Option<String> {
    let state = parse_step(
        step,
        DeleteAccountState {
            name: String::new(),
            account_name: None,
        },
        |reader, event, state| match event.name().as_ref() {
            b"Step" => set_step_name(&mut state.name, event),
            b"Parameter" if attribute_matches(event, "type", "Calculation") => {
                state.account_name = parse_calculation(reader, event);
            }
            _ => {}
        },
    );

    let mut parts = Vec::new();
    push_labeled_part(&mut parts, "Account Name", state.account_name);
    format_step(state.name, parts)
}

pub fn sanitize_enable_account(step: &str) -> Option<String> {
    let state = parse_step(
        step,
        EnableAccountState {
            name: String::new(),
            account_name: None,
            activate: None,
        },
        |reader, event, state| match event.name().as_ref() {
            b"Step" => set_step_name(&mut state.name, event),
            b"Boolean" if attribute_matches(event, "type", "enable") => {
                state.activate = Some(attribute_matches(event, "value", "True"));
            }
            b"Parameter" if attribute_matches(event, "type", "Calculation") => {
                state.account_name = parse_calculation(reader, event);
            }
            _ => {}
        },
    );

    let mut parts: Vec<String> = Vec::new();
    push_labeled_part(&mut parts, "Account Name", state.account_name);
    if let Some(act) = state.activate {
        parts.push(if act {
            "Activate".to_string()
        } else {
            "Deactivate".to_string()
        });
    }

    format_step(state.name, parts)
}

pub fn sanitize_re_login(step: &str, obfuscate: bool) -> Option<String> {
    let state = parse_step(
        step,
        ReLoginState {
            name: String::new(),
            account_name: None,
            password: None,
            with_dialog: true,
        },
        |reader, event, state| match event.name().as_ref() {
            b"Step" => set_step_name(&mut state.name, event),
            b"Boolean" => {
                if attribute_matches(event, "type", "With dialog") {
                    state.with_dialog = attribute_matches(event, "value", "True");
                }
            }
            b"Parameter" => match get_attribute(event, "type").as_deref() {
                Some("Name") => state.account_name = parse_calculation(reader, event),
                Some("Password") => state.password = parse_secret(reader, event, obfuscate),
                _ => {}
            },
            _ => {}
        },
    );

    let mut parts: Vec<String> = Vec::new();
    push_labeled_part(&mut parts, "Account Name", state.account_name);
    push_labeled_part(&mut parts, "Password", state.password);
    parts.push(format!("With dialog: {}", on_off(state.with_dialog)));

    format_step(state.name, parts)
}

pub fn sanitize_reset_account_password(step: &str, obfuscate: bool) -> Option<String> {
    let state = parse_step(
        step,
        ResetAccountPasswordState {
            name: String::new(),
            account_name: None,
            password: None,
            expire_password: false,
        },
        |reader, event, state| match event.name().as_ref() {
            b"Step" => set_step_name(&mut state.name, event),
            b"Boolean" => {
                if attribute_matches(event, "type", "Password")
                    && attribute_matches(event, "value", "True")
                {
                    state.expire_password = true;
                }
            }
            b"Parameter" => match get_attribute(event, "type").as_deref() {
                Some("Name") => state.account_name = parse_calculation(reader, event),
                Some("Password") => state.password = parse_secret(reader, event, obfuscate),
                _ => {}
            },
            _ => {}
        },
    );

    let mut parts: Vec<String> = Vec::new();
    push_labeled_part(&mut parts, "Account Name", state.account_name);
    push_labeled_part(&mut parts, "Password", state.password);
    push_flag(&mut parts, state.expire_password, "Expire password");

    format_step_with_empty_brackets(state.name, parts)
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
