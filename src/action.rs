use std::path::PathBuf;

pub enum Action {
    Quit,
    CursorUp,
    CursorDown,
    CursorToFirst,
    CursorToLast,
    SwitchSrc,
    ChangeDir(PathBuf),
    ChangeDirToParent(PathBuf),
    StartSearch,
    EndSearch,
    Search(String),
}
