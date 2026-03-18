use crate::Skeleton;
use crate::utils::xml_utils::XmlEventType;

/// Push a line to the skeleton
/// But do this so that <tag>value</tag> is pushed as a single line instead of <tag>\nvalue\n</tag>
pub fn push_line_to_skeleton(
    skeleton: &mut Skeleton,
    base_depth: usize,
    relative_depth: usize,
    str_to_push: &str,
    is_child_start_tag: bool,
    current_event_type: XmlEventType,
) {
    if str_to_push.is_empty() {
        return;
    }

    let depth = base_depth + relative_depth - 1 - usize::from(is_child_start_tag);
    let indent = "\t".repeat(depth);
    let line = format!("{indent}{str_to_push}");

    if matches!(
        current_event_type,
        XmlEventType::Start | XmlEventType::Other
    ) {
        if skeleton.previous_event_type == XmlEventType::Start {
            skeleton.content.push_str(&skeleton.previous_line);
            skeleton.content.push('\n');
        }
        skeleton.previous_line.clear();
        skeleton.previous_line.push_str(&line);
    } else {
        // For non-start events (End, Text, CData, Comment), flush any pending line and
        // determine whether to inline (trim) the current content onto the same line.
        // This enables compact output like `<tag>value</tag>` on a single line.
        let has_pending = !skeleton.previous_line.is_empty();
        if has_pending {
            skeleton.content.push_str(&skeleton.previous_line);
            skeleton.previous_line.clear();
        }

        // Decide whether to inline (trim whitespace) or emit on its own line.
        // Inline when: continuing a Start tag (e.g., <tag>value</tag>),
        // but not when an "Other" element precedes an End tag.
        let is_end_after_other = current_event_type == XmlEventType::End
            && skeleton.previous_event_type == XmlEventType::Other;
        let inline = if has_pending {
            if is_end_after_other {
                skeleton.content.push('\n');
                false
            } else {
                true
            }
        } else {
            current_event_type == XmlEventType::End
                && skeleton.previous_event_type != XmlEventType::End
        };

        if inline {
            skeleton.content.push_str(line.trim());
        } else {
            skeleton.content.push_str(&line);
        }
        if current_event_type == XmlEventType::End {
            skeleton.content.push('\n');
        }
    }
    skeleton.previous_event_type = current_event_type;
}
