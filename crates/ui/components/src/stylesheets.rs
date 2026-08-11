use dioxus::prelude::*;

const BADGE_STYLESHEET: Asset = asset!("/src/badge/style.css", AssetOptions::css());
const BUTTON_STYLESHEET: Asset = asset!("/src/button/style.css", AssetOptions::css());
const CHECKBOX_STYLESHEET: Asset = asset!("/src/checkbox/style.css", AssetOptions::css());
const COLLECTION_TREE_STYLESHEET: Asset =
    asset!("/src/collection_tree/style.css", AssetOptions::css());
const DATA_TABLE_STYLESHEET: Asset = asset!("/src/data_table/style.css", AssetOptions::css());
const DIALOG_STYLESHEET: Asset = asset!("/src/dialog/style.css", AssetOptions::css());
const INPUT_STYLESHEET: Asset = asset!("/src/input/style.css", AssetOptions::css());
const TEXTAREA_STYLESHEET: Asset = asset!("/src/textarea/style.css", AssetOptions::css());

/// 加载后台组件使用的稳定样式资源。
#[component]
pub fn UiStylesheets() -> Element {
    rsx! {
        document::Stylesheet { href: BADGE_STYLESHEET }
        document::Stylesheet { href: BUTTON_STYLESHEET }
        document::Stylesheet { href: CHECKBOX_STYLESHEET }
        document::Stylesheet { href: COLLECTION_TREE_STYLESHEET }
        document::Stylesheet { href: DATA_TABLE_STYLESHEET }
        document::Stylesheet { href: DIALOG_STYLESHEET }
        document::Stylesheet { href: INPUT_STYLESHEET }
        document::Stylesheet { href: TEXTAREA_STYLESHEET }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn stylesheets_keep_stable_component_classes() {
        let stylesheets = [
            (include_str!("badge/style.css"), ".dx-badge"),
            (include_str!("button/style.css"), ".dx-button"),
            (include_str!("checkbox/style.css"), ".dx-checkbox"),
            (
                include_str!("collection_tree/style.css"),
                ".collection-tree",
            ),
            (include_str!("data_table/style.css"), ".data-table-root"),
            (include_str!("dialog/style.css"), ".dx-dialog"),
            (include_str!("input/style.css"), ".dx-input"),
            (include_str!("textarea/style.css"), ".dx-textarea"),
        ];

        for (stylesheet, class_name) in stylesheets {
            assert!(
                stylesheet.contains(class_name),
                "样式资源缺少稳定类名：{class_name}"
            );
        }
    }
}
