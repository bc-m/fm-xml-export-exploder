//! Send Mail.

use crate::script_steps::parameters::files::universal_path_list;
use crate::script_steps::parameters::format::{labeled, on_off};
use crate::utils::xml_node::Node;

/// Send Mail. Passwords and private keys are left out on purpose.
pub(crate) fn email(node: &Node) -> Vec<String> {
    let mut parts = Vec::new();
    for boolean in node.children_named("Boolean") {
        if let Some(label) = boolean.attr("type") {
            parts.push(format!("{label}: {}", on_off(boolean, "value")));
        }
    }

    let send = node.child("Send");
    let via = node
        .child("SMTP")
        .map(|smtp| ("Send via SMTP Server", smtp))
        .or_else(|| {
            node.child("OAuthAuthentication")
                .map(|oauth| ("Send via OAuth", oauth))
        });
    match via {
        Some((label, account)) => {
            parts.push(label.to_string());
            for child in &account.children {
                let label = match child.name.as_str() {
                    "Name" => "From name",
                    "Email" => "From address",
                    "ReplyTo" => "Reply-to",
                    "Server" => "SMTP server",
                    "Port" => "Port",
                    "UserName" => "User name",
                    "UserID" => "User ID",
                    "ServiceAccount" => "Service account",
                    "Encryption" | "Authentication" | "OAuthProvider" => {
                        let label = match child.name.as_str() {
                            "OAuthProvider" => "OAuth provider",
                            name => name,
                        };
                        parts.extend(labeled(label, child.attr("name").map(str::to_string)));
                        continue;
                    }
                    _ => continue,
                };
                parts.extend(labeled(label, child.calculation()));
            }
        }
        None if send.is_some() => parts.push("Send via E-mail Client".to_string()),
        None => {}
    }

    if let Some(send) = send {
        if send
            .child("Multiple")
            .is_some_and(|m| m.attr_is_true("value"))
        {
            parts.push("Multiple emails (one for each record in found set)".to_string());
        }
        for (child, label) in [
            ("To", "To"),
            ("CC", "CC"),
            ("BCC", "BCC"),
            ("Subject", "Subject"),
            ("Message", "Message"),
        ] {
            parts.extend(labeled(
                label,
                send.child(child).and_then(Node::calculation),
            ));
        }
    }

    let attachments = universal_path_list(node);
    if let Some(paths) = attachments.first() {
        parts.push(format!("Attachments: {paths}"));
    }
    parts
}

#[cfg(test)]
mod tests {
    use crate::script_steps::parameters::parameter_values::parse;

    #[test]
    fn email_smtp() {
        let calc = |text: &str| {
            format!(
                r#"<Calculation datatype="1" position="0"><Calculation><Text><![CDATA[{text}]]></Text></Calculation></Calculation>"#
            )
        };
        let xml = format!(
            r#"<Parameter type="Email"><Boolean type="With dialog" value="False"></Boolean><SMTP><Name>{}</Name><Server>{}</Server><Encryption name="TLS" value="2"></Encryption><Password>{}</Password></SMTP><Send OAuthAuthentication="False" SMTP="True"><Multiple value="False"></Multiple><To><CollectAddresses value="False"></CollectAddresses>{}</To><Subject>{}</Subject></Send><UniversalPathList membercount="1"><ObjectList><Location><![CDATA[$file]]></Location></ObjectList></UniversalPathList></Parameter>"#,
            calc("$name"),
            calc("$smtp"),
            calc("$secret"),
            calc("$to"),
            calc("$subject"),
        );
        assert_eq!(
            parse(&xml, 0).as_deref(),
            Some(
                r#"With dialog: OFF ; Send via SMTP Server ; From name: $name ; SMTP server: $smtp ; Encryption: TLS ; To: $to ; Subject: $subject ; Attachments: "$file""#
            )
        );
    }
}
