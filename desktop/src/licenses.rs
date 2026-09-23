// ponytail: reviewed notices snapshot; update when dependencies or the Rust toolchain change.
pub const LICENSES: &str = concat!(
    "waid - Licenses and third-party notices\n\n",
    include_str!("../../LICENSE"),
    "\n\n",
    include_str!("../../THIRD_PARTY_NOTICES.txt"),
    "\n\nPretendard font license\n=======================\n\n",
    include_str!("../assets/fonts/LICENSE.txt"),
);
