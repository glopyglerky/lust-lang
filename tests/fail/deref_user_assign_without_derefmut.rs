// expect: L0309
// rustc E0594: assigning through a user `Deref` needs `DerefMut`.
use std::ops::Deref;
struct Late { file_index: usize }
struct Info { late: Late }
impl Deref for Info { type Target = Late; fn deref(&self) -> &Late { &self.late } }
fn main() { let mut info = Info { late: Late { file_index: 3 } }; info.file_index = 4; }
