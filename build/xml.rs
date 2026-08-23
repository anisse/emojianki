use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

pub(crate) enum ParseEvent {
    Start(HashMap<String, String>),
    Text(String),
}
// TODO: remove
pub(crate) fn parse_xml_streaming(
    s: &str,
    callback_path: &[&str],
    mut cb_fn: impl FnMut(ParseEvent),
) {
    parse_xml_multiple(
        s,
        &mut [Matcher {
            path: callback_path,
            cb_fn: &mut cb_fn,
        }],
    )
}
pub(crate) struct Matcher<'a> {
    pub(crate) path: &'a [&'a str],
    pub(crate) cb_fn: &'a mut dyn FnMut(ParseEvent),
}
pub(crate) fn parse_xml_multiple<'a>(s: &'a str, matches: &mut [Matcher<'a>]) {
    let mut reader = Reader::from_str(s);
    reader.config_mut().trim_text(true);

    let mut path = vec![];
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
            Ok(Event::Empty(e)) => {
                process_start_event(e, &mut path, matches);
                path.pop();
            }
            Ok(Event::Start(e)) => process_start_event(e, &mut path, matches),
            Ok(Event::Text(e)) => {
                for Matcher {
                    path: allowed_path,
                    cb_fn,
                } in matches.iter_mut()
                {
                    if allowed_path == &path {
                        let text = e
                            .decode()
                            .expect("utf-8 content in text of tag")
                            .into_owned();
                        cb_fn(ParseEvent::Text(text));
                    }
                }
            }
            _ => (),
        }
    }
}

fn process_start_event<'a>(e: BytesStart<'a>, path: &mut Vec<String>, matches: &mut [Matcher<'a>]) {
    let tag_name = str::from_utf8(e.name().as_ref())
        .expect("utf-8 tag name")
        .to_string(); // alloc gallore
    path.push(tag_name);
    for Matcher {
        path: allowed_path,
        cb_fn,
    } in matches.iter_mut()
    {
        if allowed_path == path {
            let attrs = e
                .attributes()
                .map(|a| {
                    let attr = a.expect("tag should have attributes");
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
            cb_fn(ParseEvent::Start(attrs));
        }
    }
}
