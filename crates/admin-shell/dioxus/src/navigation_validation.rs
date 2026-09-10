use std::collections::{HashMap, HashSet};

use anyhow::{Result, ensure};

use crate::ApplicationMenuGroup;

pub(crate) struct NavigationContribution<'a> {
    pub(crate) page_id: &'a str,
    pub(crate) page_label: &'a str,
    pub(crate) scene_id: &'a str,
    pub(crate) scene_label: &'a str,
    pub(crate) menu_path: &'a [ApplicationMenuGroup],
    pub(crate) sidebar: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GroupLocation {
    scene_id: String,
    parent_id: Option<String>,
    label: String,
    icon: Option<String>,
}

pub(crate) fn validate_navigation_contributions<'a>(
    contributions: impl IntoIterator<Item = NavigationContribution<'a>>,
) -> Result<()> {
    let mut page_ids = HashSet::new();
    let mut scenes = HashMap::<&str, &str>::new();
    let mut groups = HashMap::<&str, GroupLocation>::new();

    for contribution in contributions {
        ensure!(!contribution.page_id.trim().is_empty(), "页面 id 不能为空");
        ensure!(
            !contribution.page_label.trim().is_empty(),
            "页面标题不能为空"
        );
        ensure!(
            page_ids.insert(contribution.page_id),
            "页面 id 重复: {}",
            contribution.page_id
        );
        if !contribution.sidebar {
            continue;
        }

        ensure!(!contribution.scene_id.trim().is_empty(), "场景 id 不能为空");
        ensure!(
            !contribution.scene_label.trim().is_empty(),
            "场景标题不能为空"
        );
        if let Some(label) = scenes.insert(contribution.scene_id, contribution.scene_label) {
            ensure!(
                label == contribution.scene_label,
                "同一场景 id 的标题不一致: {}",
                contribution.scene_id
            );
        }

        let mut path_ids = HashSet::new();
        let mut parent_id = None::<String>;
        for group in contribution.menu_path {
            validate_group(group)?;
            ensure!(
                path_ids.insert(group.id.as_str()),
                "菜单分组路径形成循环: {} -> {}",
                contribution.page_id,
                group.id
            );
            let location = GroupLocation {
                scene_id: contribution.scene_id.to_owned(),
                parent_id: parent_id.clone(),
                label: group.label.clone(),
                icon: group.icon.clone(),
            };
            if let Some(existing) = groups.get(group.id.as_str()) {
                ensure!(
                    existing == &location,
                    "同一菜单分组 id 的场景、父节点或展示信息不一致: {}",
                    group.id
                );
            } else {
                groups.insert(group.id.as_str(), location);
            }
            parent_id = Some(group.id.clone());
        }
    }

    for group_id in groups.keys() {
        ensure!(
            !page_ids.contains(group_id),
            "菜单分组 id 与页面 id 冲突: {group_id}"
        );
    }
    Ok(())
}

fn validate_group(group: &ApplicationMenuGroup) -> Result<()> {
    ensure!(!group.id.trim().is_empty(), "菜单分组 id 不能为空");
    ensure!(!group.label.trim().is_empty(), "菜单分组标题不能为空");
    ensure!(
        group
            .icon
            .as_deref()
            .is_none_or(|icon| !icon.trim().is_empty()),
        "菜单分组图标不能为空字符串"
    );
    Ok(())
}
