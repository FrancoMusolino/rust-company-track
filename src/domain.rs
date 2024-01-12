use std::error::Error;

use crate::Database;

pub trait AggregateRoot<Event> {
    fn apply(&mut self, event: Event) -> ();
    fn commit(&mut self) -> ();
    fn get_uncommited_events(&self) -> &Vec<DomainEvent<Event>>;
}

pub trait Repository<E, A: AggregateRoot<E>> {
    fn get(db: &Database) -> Result<A, Box<dyn Error>>;
    fn save(&self, db: &Database) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug)]
pub struct DomainEvent<T> {
    pub event: T,
}
