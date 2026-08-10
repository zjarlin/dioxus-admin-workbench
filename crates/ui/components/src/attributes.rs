use dioxus::core::AttributeValue;
use dioxus::prelude::Attribute;

pub(crate) fn with_class(
    mut attributes: Vec<Attribute>,
    class: impl Into<String>,
) -> Vec<Attribute> {
    let class = class.into();
    let existing = attributes
        .iter_mut()
        .find(|attribute| attribute.name == "class" && attribute.namespace.is_none());

    if let Some(existing) = existing {
        let AttributeValue::Text(current) = &existing.value else {
            existing.value = AttributeValue::Text(class);
            return attributes;
        };
        existing.value = AttributeValue::Text(join_classes(&class, current));
        return attributes;
    }

    attributes.push(Attribute {
        name: "class",
        value: AttributeValue::Text(class),
        namespace: None,
        volatile: false,
    });
    attributes
}

fn join_classes(left: &str, right: &str) -> String {
    let left = left.trim();
    let right = right.trim();
    match (left.is_empty(), right.is_empty()) {
        (false, false) => format!("{left} {right}"),
        (false, true) => left.to_owned(),
        (true, false) => right.to_owned(),
        (true, true) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(value: &str) -> Attribute {
        Attribute {
            name: "class",
            value: AttributeValue::Text(value.to_owned()),
            namespace: None,
            volatile: false,
        }
    }

    #[test]
    fn component_class_precedes_consumer_class() {
        let attributes = with_class(vec![class("consumer")], "component");
        let AttributeValue::Text(value) = &attributes[0].value else {
            panic!("class 应为文本属性");
        };
        assert_eq!(value, "component consumer");
    }
}
