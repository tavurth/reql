extern crate futures;
extern crate tokio_core;

#[macro_use]
extern crate reql;
extern crate reql_types;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use reql_types::Change;
use futures::stream::Stream;
use tokio_core::reactor::Core;
use reql::{Client, Document, Run};

/**
 * rethinkdb changelog example
 * After running, access your admin panel at localhost:8080
 *
 * setup a "test" database, with a "test" table, and run:
 *
 * // Insert an item
 * r.db('test').table('test').insert({ test: 1 });
 *
 * // Give the first item a random number
 * r.db('test').table('test').nth(0).update({ test: r.random(0, 100) }, { nonAtomic: true });
 *
 * // Remove the first item
 * r.db('test').table('test').nth(0).delete();
 */

/**
 * Or, in another rust context, you could run the following:
 *
 * // Insert an item
 * r.db("test").table("test").insert(json!({ test: 1 })) // => run & unwrap
 *
 * // Give the first item a random number
 * r.db("test").table("test").nth(0).update(args!(
 *     { test: r.random(0, 100) },
 *     { nonAtomic: true }
 * ))  // => run & unwrap
 *
 * // Remove the first item
 * r.db("test").table("test").nth(0).delete() // => run & unwrap
 *
 */

#[derive(Debug, Serialize, Deserialize)]
struct TestItem {
    test: i32,
    id: String,
}

fn main()
{
    // Create a new ReQL client
    let r = Client::new();

    // Create an even loop
    let core = Core::new().unwrap();

    // Create a connection pool
    let conn = r.connect(&core.handle()).unwrap();

    // Run the query
    let query =
        r.db("test")
        .table("test")
        .filter(args!(|doc| {

            // Filter only documents which match our current TestItem trait
            doc.has_fields("test").and(doc.get_field("test").type_of().eq("NUMBER"))
        }))
        .changes()

    // We want rethinkdb to inform us of the change type
        .with_args(args!({
            include_types: true
        }))
        .run::<Change<TestItem, TestItem>>(conn)
        .unwrap();

    // Process the results
    let stati = query.and_then(|change| {
        match change {
            // The server returned the response we were expecting,
            // and deserialized the data into our Change structure
            Some(Document::Expected(change)) => {

                // Valid String change type
                if let Some(action) = change.result_type {


                    // Extract the change type
                    print!("{:?} action received\n\t=> ", action);

                    // Match the change type
                    match action.as_str() {
                        "add" => println!("{:?}", change.new_val),
                        "remove" => println!("{:?}", change.old_val),
                        "change" => println!("from {:?} to {:?}", change.old_val, change.new_val),

                        _ => println!("Unsupported change type: {:?}", action)
                    }
                } else {
                    println!("Invalid change type");
                }
            }

            // We got a response alright, but it wasn't the one we were
            // expecting plus it's not an error either, otherwise it would
            // have been returned as such (This simply means that the response
            // we got couldn't be serialised into the type we were expecting)
            Some(Document::Unexpected(change)) => {
                println!("Got unexpected change: {}",change)
            }
            // This is impossible in this particular example since there
            // needs to be at least one server available to give this
            // response otherwise we would have run into an error for
            // failing to connect
            None => {
                println!("got no documents in the database");
            }
        }
        Ok(())
    })
    // Our query ran into an error
        .or_else(|error| {
            println!("{:?}", error);
            Err(())
        });

    // Wait for all the results to be processed
    for _ in stati.wait() {}
}
