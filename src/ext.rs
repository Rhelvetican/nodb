use serde::Serialize;

use crate::nodb::NoDb;

/// A struct for extending NoDb lists and adding more items to them.
pub struct NoDbExt<'a> {
    pub(crate) db: &'a mut NoDb,
    pub(crate) list_name: String,
}

impl<'a> NoDbExt<'a> {
    /// Add a single item to an existing list.
    ///
    /// As mentioned before, the lists are heterogeneous, meaning a single list can contain
    /// items of different types. That means that the item can be of any type that is serializable.
    /// That includes all primitive types, vectors, tuples and every struct that has the
    /// `#[derive(Serialize, Deserialize)` attribute.
    /// The method returns another `NoDbExt` object that enables to continue adding
    /// items to the list.

    pub fn ladd<V: Serialize>(&mut self, value: V) -> Option<NoDbExt> {
        self.db.list_add(&self.list_name, &value)
    }

    /// Add multiple items to an existing list.
    ///
    /// As mentioned before, the lists are heterogeneous, meaning a single list can contain
    /// items of different types. That means that the item can be of any type that is serializable.
    /// That includes all primitive types, vectors, tuples and every struct that has the
    /// `#[derive(Serialize, Deserialize)` attribute.
    /// This method adds multiple items to the list, but since they're in a vector that means all
    /// of them are of the same type. Of course it doesn't mean that the list cannot contain items
    /// of other types as well, as you can see in the example below.
    /// The method returns another `NoDbExt` object that enables to continue adding
    /// items to the list.

    pub fn lextend<'b, V, I>(&mut self, seq: I) -> Option<NoDbExt>
    where
        V: 'b + Serialize,
        I: IntoIterator<Item = &'b V>,
    {
        self.db.list_extend(&self.list_name, seq)
    }
}
