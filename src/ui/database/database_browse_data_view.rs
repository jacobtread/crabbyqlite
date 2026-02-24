// use crate::database::DatabaseTable;

// pub struct DatabaseBrowseDataView {
//     tables: Vec<DatabaseTable>,
// }

// impl DatabaseBrowseDataView {
//     pub fn new(window: &mut Window, cx: &mut App) -> Entity<Self> {
//         cx.new(|cx| {
//             let app = cx.global::<AppState>();
//             let database_store = app.database_store.clone();

//             cx.subscribe_in(
//                 &database_store,
//                 window,
//                 |this: &mut DatabaseTablesView, database_store, event, window, cx| match event {
//                     DatabaseStoreEvent::DatabaseChanged => {
//                         this.load_database_tables(database_store, window, cx);
//                     }
//                 },
//             )
//             .detach();

//             Self {
//                 tables: Vec::new(),
//                 table_state,
//                 tables_task: None,
//             }
//         })
//     }
// }
