//! Canonical structural JSON for checkpoint state, including compound-key tables.
//! Records are objects; tables use sorted `$entries` pairs; wide integers use typed decimal tags.
use serde::{
    Serialize,
    ser::{
        self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
};
use serde_json::{Map, Value};
type Error = serde_json::Error;
type Result<T> = std::result::Result<T, Error>;
pub(super) fn value<T: Serialize + ?Sized>(v: &T) -> Result<Value> {
    v.serialize(Encoder)
}
struct Encoder;
macro_rules! integer {($($name:ident:$t:ty),*)=>{$(fn $name(self,v:$t)->Result<Value>{Ok(Value::from(v))})*};}
impl ser::Serializer for Encoder {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = Sequence;
    type SerializeTuple = Sequence;
    type SerializeTupleStruct = Sequence;
    type SerializeTupleVariant = Sequence;
    type SerializeMap = Table;
    type SerializeStruct = Record;
    type SerializeStructVariant = Record;
    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }
    integer!(serialize_i8:i8,serialize_i16:i16,serialize_i32:i32,serialize_u8:u8,serialize_u16:u16,serialize_u32:u32);
    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(tag("$i64", Value::String(v.to_string())))
    }
    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(tag("$u64", Value::String(v.to_string())))
    }
    fn serialize_f32(self, v: f32) -> Result<Value> {
        self.serialize_f64(v as f64)
    }
    fn serialize_f64(self, v: f64) -> Result<Value> {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| ser::Error::custom("non_finite_state_value"))
    }
    fn serialize_char(self, v: char) -> Result<Value> {
        self.serialize_str(&v.to_string())
    }
    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::String(v.into()))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        use base64::Engine as _;
        Ok(serde_json::json!({"$bytes":base64::engine::general_purpose::STANDARD.encode(v)}))
    }
    fn serialize_none(self) -> Result<Value> {
        Ok(Value::Null)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<Value> {
        value(v)
    }
    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Value> {
        Ok(Value::Null)
    }
    fn serialize_unit_variant(self, _: &'static str, _: u32, name: &'static str) -> Result<Value> {
        self.serialize_str(name)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<Value> {
        value(v)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        name: &'static str,
        v: &T,
    ) -> Result<Value> {
        Ok(tag(name, value(v)?))
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Sequence> {
        Ok(Sequence {
            values: vec![],
            variant: None,
        })
    }
    fn serialize_tuple(self, n: usize) -> Result<Sequence> {
        self.serialize_seq(Some(n))
    }
    fn serialize_tuple_struct(self, _: &'static str, n: usize) -> Result<Sequence> {
        self.serialize_seq(Some(n))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        name: &'static str,
        _: usize,
    ) -> Result<Sequence> {
        Ok(Sequence {
            values: vec![],
            variant: Some(name),
        })
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Table> {
        Ok(Table {
            entries: vec![],
            key: None,
        })
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Record> {
        Ok(Record {
            fields: Map::new(),
            variant: None,
        })
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        name: &'static str,
        _: usize,
    ) -> Result<Record> {
        Ok(Record {
            fields: Map::new(),
            variant: Some(name),
        })
    }
}
fn tag(name: &str, v: Value) -> Value {
    Value::Object(Map::from_iter([(name.into(), v)]))
}
struct Sequence {
    values: Vec<Value>,
    variant: Option<&'static str>,
}
impl Sequence {
    fn push<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        self.values.push(value(v)?);
        Ok(())
    }
    fn finish(self) -> Value {
        let v = Value::Array(self.values);
        if let Some(name) = self.variant {
            tag(name, v)
        } else {
            v
        }
    }
}
impl SerializeSeq for Sequence {
    type Ok = Value;
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        self.push(v)
    }
    fn end(self) -> Result<Value> {
        Ok(self.finish())
    }
}
impl SerializeTuple for Sequence {
    type Ok = Value;
    type Error = Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        self.push(v)
    }
    fn end(self) -> Result<Value> {
        Ok(self.finish())
    }
}
impl SerializeTupleStruct for Sequence {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        self.push(v)
    }
    fn end(self) -> Result<Value> {
        Ok(self.finish())
    }
}
impl SerializeTupleVariant for Sequence {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        self.push(v)
    }
    fn end(self) -> Result<Value> {
        Ok(self.finish())
    }
}
struct Table {
    entries: Vec<(Value, Value)>,
    key: Option<Value>,
}
impl SerializeMap for Table {
    type Ok = Value;
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Result<()> {
        if self.key.is_some() {
            return Err(ser::Error::custom("state_map_key_without_value"));
        }
        self.key = Some(value(k)?);
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        self.entries.push((
            self.key
                .take()
                .ok_or_else(|| ser::Error::custom("state_map_value_without_key"))?,
            value(v)?,
        ));
        Ok(())
    }
    fn end(mut self) -> Result<Value> {
        if self.key.is_some() {
            return Err(ser::Error::custom("state_map_incomplete"));
        }
        self.entries
            .sort_by_cached_key(|(key, _)| serde_json::to_vec(key).expect("JSON key"));
        Ok(tag(
            "$entries",
            Value::Array(
                self.entries
                    .into_iter()
                    .map(|(k, v)| Value::Array(vec![k, v]))
                    .collect(),
            ),
        ))
    }
}
struct Record {
    fields: Map<String, Value>,
    variant: Option<&'static str>,
}
impl Record {
    fn field<T: Serialize + ?Sized>(&mut self, k: &'static str, v: &T) -> Result<()> {
        self.fields.insert(k.into(), value(v)?);
        Ok(())
    }
    fn finish(self) -> Value {
        let v = Value::Object(self.fields);
        if let Some(name) = self.variant {
            tag(name, v)
        } else {
            v
        }
    }
}
impl SerializeStruct for Record {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, k: &'static str, v: &T) -> Result<()> {
        self.field(k, v)
    }
    fn end(self) -> Result<Value> {
        Ok(self.finish())
    }
}
impl SerializeStructVariant for Record {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, k: &'static str, v: &T) -> Result<()> {
        self.field(k, v)
    }
    fn end(self) -> Result<Value> {
        Ok(self.finish())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compound_keys_and_large_counters_are_lossless_and_ordered() {
        let map =
            std::collections::BTreeMap::from([(("nic-b", u64::MAX), 42u64), (("nic-a", 1), 9)]);
        let json = value(&map).unwrap();
        assert_eq!(
            json["$entries"][0][0],
            serde_json::json!(["nic-a", {"$u64":"1"}])
        );
        assert_eq!(json["$entries"][1][0][1]["$u64"], u64::MAX.to_string());
        assert_eq!(json["$entries"][1][1]["$u64"], "42");
    }
}
