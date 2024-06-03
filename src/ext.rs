use serde::Serialize;

use crate::nodb::NoDb;

pub struct NoDbExt<'a> {
    pub(crate) db: &'a mut NoDb,
    pub(crate) list_name: String,
}

impl<'a> NoDbExt<'a> {
    pub fn ladd<V: Serialize>(&mut self, value: V) -> Option<NoDbExt> {
        self.db.ladd(&self.list_name, &value)
    }
    pub fn lextend<'b, V, I>(&mut self, seq: I) -> Option<NoDbExt>
    where
        V: 'b + Serialize,
        I: IntoIterator<Item = &'b V>,
    {
        self.db.lextend(&self.list_name, seq)
    }
}
