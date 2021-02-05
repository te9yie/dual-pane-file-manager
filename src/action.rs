use std::path::PathBuf;

pub enum Action {
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
}
