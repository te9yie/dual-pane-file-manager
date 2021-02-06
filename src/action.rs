use std::path::PathBuf;

pub enum Action {
    Refresh,
    Quit,
    CursorUp,
    CursorDown,
    CursorToFirst,
    CursorToLast,
    ToggleMark,
    SwitchSrc,
    ChangeDir(PathBuf),
    ChangeDirToParent(PathBuf),
    Execute(PathBuf),
    Edit(PathBuf),
    StartSearch,
    EndSearch,
    Search(String),
    Copy,
    Move,
    Delete,
    StartCreateDir,
    EndInputText(Option<String>),
}
