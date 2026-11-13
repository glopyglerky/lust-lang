use crate::astdata::text::JsString;

const ESCAPE: char = '\u{10fffd}';
const SURROGATE_PAYLOAD_START: u32 = 0xe000;
const FIRST_SURROGATE: u16 = 0xd800;
const LAST_SURROGATE: u16 = 0xdfff;





#[cfg(test)]
mod tests {
    use crate::astdata::text::JsString;

    use super::{
        ESCAPE, append_js_string, decode_js_string, encode_js_string, normalize_js_string,
    };

    #[test]
    fn ordinary_unicode_text_keeps_its_existing_ast_representation() {
        let original = JsString::from_utf8("plain π 😀 text");
        let encoded = encode_js_string(&original);

        assert_eq!(encoded, "plain π 😀 text");
        assert_eq!(decode_js_string(&encoded), original);
    }

    #[test]
    fn distinct_lone_surrogates_remain_distinct_and_lossless() {
        let high = JsString::from_units(vec![0xd800]);
        let low = JsString::from_units(vec![0xdc00]);
        let high_encoded = encode_js_string(&high);
        let low_encoded = encode_js_string(&low);

        assert_ne!(high_encoded, low_encoded);
        assert_eq!(decode_js_string(&high_encoded).as_units(), &[0xd800]);
        assert_eq!(decode_js_string(&low_encoded).as_units(), &[0xdc00]);
    }

    #[test]
    fn surrogate_pairs_combine_only_when_their_code_units_touch() {
        let high = encode_js_string(&JsString::from_units(vec![0xd83d]));
        let low = encode_js_string(&JsString::from_units(vec![0xde00]));

        let mut adjacent = high.clone();
        append_js_string(&mut adjacent, &low);
        assert_eq!(adjacent, "😀");
        assert_eq!(decode_js_string(&adjacent).as_units(), &[0xd83d, 0xde00]);

        let mut separated = high;
        append_js_string(&mut separated, "-");
        append_js_string(&mut separated, &low);
        assert_eq!(
            decode_js_string(&separated).as_units(),
            &[0xd83d, u16::from(b'-'), 0xde00]
        );
        assert_ne!(separated, "😀");
    }

    #[test]
    fn real_private_use_characters_and_escape_markers_are_unambiguous() {
        let original = JsString::from_units(vec![0xdbff, 0xdffd, 0xe000, 0xd800, 0xdbff, 0xdffd]);
        let encoded = encode_js_string(&original);

        assert_eq!(decode_js_string(&encoded), original);
        assert_eq!(normalize_js_string(&encoded), encoded);
        assert_eq!(
            encode_js_string(&JsString::from_utf8(&ESCAPE.to_string())),
            {
                let mut escaped = ESCAPE.to_string();
                escaped.push(ESCAPE);
                escaped
            }
        );
    }
}
