pub use sea_orm_migration::prelude::*;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250306_130625_init::Migration),
            Box::new(m20250309_125602_user_data_update::Migration),
        ]
    }
}

mod m20250306_130625_init;
mod m20250309_125602_user_data_update;
