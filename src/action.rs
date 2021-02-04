use std::path::PathBuf;

pub enum Action {
    Quit,
    CursorUp,
    CursorDown,
    SwitchSrc,
    ChangeDir(PathBuf),
    ChangeDirToParent(PathBuf),
}
