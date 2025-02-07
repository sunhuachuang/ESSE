#[rustfmt::skip]
pub(super) const GROUP_VERSIONS: [&str; 3] = [
  "CREATE TABLE IF NOT EXISTS groups(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    height INTEGER NOT NULL,
    gcd TEXT NOT NULL,
    addr TEXT NOT NULL,
    name TEXT NOT NULL,
    is_close INTEGER NOT NULL,
    is_local INTEGER NOT NULL);",
  "CREATE TABLE IF NOT EXISTS members(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    height INTEGER NOT NULL,
    fid INTEGER NOT NULL,
    mid TEXT NOT NULL,
    addr TEXT NOT NULL,
    name TEXT NOT NULL,
    leave INTEGER NOT NULL);",
  "CREATE TABLE IF NOT EXISTS messages(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    height INTEGER NOT NULL,
    fid INTEGER NOT NULL,
    mid INTEGER NOT NULL,
    is_me INTEGER NOT NULL,
    m_type INTEGER NOT NULL,
    content TEXT NOT NULL,
    is_delivery INTEGER NOT NULL,
    datetime INTEGER NOT NULL);",
];
