/*
 * Adapted from example in issue thread here: https://github.com/serde-rs/serde/issues/1028
 */
use serde::{Serialize, Deserialize};
use serde::{Serializer, Deserializer};
use serde_json::Value;
use serde_json::json;
use serde::de::Error;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum MyRequests {
    ItemAtIndex { index: usize },
    Length,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum MyNotifications {
    InsertItem { item: Value, index: usize },
    ShrinkToFit { }
}

#[derive(Debug)]
enum JsonRpc<N, R> {
    Request(usize, R),
    Notification(N),
}

impl<N, R> Serialize for JsonRpc<N, R>
    where N: Serialize,
          R: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            JsonRpc::Request(id, ref r) => {
                let mut v = serde_json::to_value(r).map_err(serde::ser::Error::custom)?;
                v["id"] = json!(id);
                v.serialize(serializer)
            }
            JsonRpc::Notification(ref n) => n.serialize(serializer),
        }
    }
}

impl<'de, N, R> Deserialize<'de> for JsonRpc<N, R>
    where N: Deserialize<'de>,
          R: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct IdHelper {
            id: Option<usize>,
        }

        let v = Value::deserialize(deserializer)?;
        let helper = IdHelper::deserialize(&v).map_err(Error::custom)?;
        match helper.id {
            Some(id) => {
                let r = R::deserialize(v).map_err(Error::custom)?;
                Ok(JsonRpc::Request(id, r))
            }
            None => {
                let n = N::deserialize(v).map_err(Error::custom)?;
                Ok(JsonRpc::Notification(n))
            }
        }
    }
}

fn main() {
    let j = r#"{
        "id": 42,
        "method": "item_at_index",
        "params": {
            "index": 5
        }
    }"#;
    let k = r#"{
        "method": "insert_item",
        "params": {
            "index": 5,
            "item": 42
        }
    }"#;
    let l = r#"{
        "method": "shrink_to_fit",
        "params": {}
    }"#;

    type T = JsonRpc<MyNotifications, MyRequests>;
    println!("{:?}", serde_json::from_str::<T>(j).unwrap());
    println!("{:?}", serde_json::from_str::<T>(k).unwrap());
    println!("{:?}", serde_json::from_str::<T>(l).unwrap());
}

