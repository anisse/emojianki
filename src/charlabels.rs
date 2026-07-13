use std::collections::HashMap;

use log::trace;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub(crate) type CharLabels = HashMap<String, String>;

pub(crate) fn parse_charlabels(s: &str) -> CharLabels {
    let mut clabels = CharLabels::new();
    let mut reader = Reader::from_str(s);
    reader.config_mut().trim_text(true);

    let mut path = vec![];
    let mut typ: Option<String> = None;
    // Hand rolled parser, not much better than DOM, but that will have to do
    loop {
        match reader.read_event() {
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            // exits the loop when reaching end of file
            Ok(Event::Eof) => break,
            Ok(Event::End(e)) => {
                let tag_qname = e.name();
                let tag_name = str::from_utf8(tag_qname.as_ref()).expect("utf-8 tag name");
                assert_eq!(&path.pop().expect("something to pop"), tag_name);
            }
            Ok(Event::Start(e)) => {
                let tag_name = str::from_utf8(e.name().as_ref())
                    .expect("utf-8 tag name")
                    .to_string(); // alloc gallore
                path.push(tag_name);
                trace!("Tag path: {path:?}");
                if path == ["ldml", "characterLabels", "characterLabel"] {
                    let mut attrs = e
                        .attributes()
                        .map(|a| {
                            let attr = a.expect("characterLabel tag should have attributes");
                            (
                                str::from_utf8(attr.key.as_ref())
                                    .expect("utf-8 str in attr key")
                                    .to_string(),
                                str::from_utf8(&(attr.value))
                                    .expect("utf-8 str in attr value")
                                    .to_string(),
                            )
                        })
                        .collect::<HashMap<_, _>>();
                    typ = Some(attrs.remove("type").expect("type should be present"));
                }
            }
            Ok(Event::Text(e)) => {
                let text = e
                    .decode()
                    .expect("utf-8 content in text of tag")
                    .into_owned();
                if let Some(label_type) = typ.take() {
                    clabels.insert(label_type, text);
                }
            }
            _ => (),
        }
    }
    clabels
}
