#[cfg(feature = "layout-cldr")]
pub mod layout_table {
    use crate::data::generate::get_layout_entry;
    use crate::CharacterDirection;
    use std::path::Path;
    use tinystr::{TinyStr4, TinyStr8};

    pub fn is_rtl(subtag: u64) -> bool {
        let path = "./data/cldr-misc-modern";

        let langid = unsafe { TinyStr8::new_unchecked(subtag) };
        let path = Path::new(path)
            .join("main")
            .join(&langid.to_string())
            .join("layout.json");

        let entry = get_layout_entry(path).expect("Entry retrival failed.");
        entry.2 == CharacterDirection::RTL
    }
}

#[cfg(feature = "likelysubtags-cldr")]
pub mod likelysubtags {
    use crate::data::generate::get_likely_subtags_data;
    use tinystr::{TinyStr4, TinyStr8};

    pub static mut DATA: Option<(
        Vec<(u64, (Option<u64>, Option<u32>, Option<u32>))>, // LANG_ONLY
        Vec<(u64, u32, (Option<u64>, Option<u32>, Option<u32>))>, // LANG_REGION
        Vec<(u64, u32, (Option<u64>, Option<u32>, Option<u32>))>, // LANG_SCRIPT
        Vec<(u32, u32, (Option<u64>, Option<u32>, Option<u32>))>, // SCRIPT_REGION
        Vec<(u32, (Option<u64>, Option<u32>, Option<u32>))>, // REGION_ONLY
        Vec<(u32, (Option<u64>, Option<u32>, Option<u32>))>, // SCRIPT_ONLY
    )> = None;

    unsafe fn get_lang_from_parts(
        input: (Option<u64>, Option<u32>, Option<u32>),
        lang: Option<TinyStr8>,
        script: Option<TinyStr4>,
        region: Option<TinyStr4>,
    ) -> Option<(Option<TinyStr8>, Option<TinyStr4>, Option<TinyStr4>)> {
        let lang = lang.or_else(|| input.0.map(|l| TinyStr8::new_unchecked(l)));
        let script = script.or_else(|| input.1.map(|s| TinyStr4::new_unchecked(s)));
        let region = region.or_else(|| input.2.map(|r| TinyStr4::new_unchecked(r)));
        Some((lang, script, region))
    }

    pub fn add_likely_subtags(
        lang: Option<TinyStr8>,
        script: Option<TinyStr4>,
        region: Option<TinyStr4>,
    ) -> Option<(Option<TinyStr8>, Option<TinyStr4>, Option<TinyStr4>)> {
        if lang.is_some() && script.is_some() && region.is_some() {
            return None;
        }

        if unsafe { DATA.is_none() } {
            let path = "./data/cldr-core";
            let (_, lang_only, lang_region, lang_script, script_region, region_only, script_only) =
                get_likely_subtags_data(path);

            unsafe {
                DATA = Some((
                    lang_only,
                    lang_region,
                    lang_script,
                    script_region,
                    region_only,
                    script_only,
                ));
            };
        }
        let data = unsafe { DATA.as_ref().unwrap() };

        if let Some(l) = lang {
            if let Some(r) = region {
                let result = data
                    .1
                    .binary_search_by_key(&(&l.into(), &r.into()), |(key_l, key_r, _)| {
                        (key_l, key_r)
                    })
                    .ok();
                if let Some(r) = result {
                    // safe because all table entries are well formed.
                    return unsafe { get_lang_from_parts(data.1[r].2, None, None, None) };
                }
            }

            if let Some(s) = script {
                let result = data
                    .2
                    .binary_search_by_key(&(&l.into(), &s.into()), |(key_l, key_s, _)| {
                        (key_l, key_s)
                    })
                    .ok();
                if let Some(r) = result {
                    // safe because all table entries are well formed.
                    return unsafe { get_lang_from_parts(data.2[r].2, None, None, None) };
                }
            }

            let result = data
                .0
                .binary_search_by_key(&(&l.into()), |(key_l, _)| key_l)
                .ok();
            if let Some(r) = result {
                // safe because all table entries are well formed.
                return unsafe { get_lang_from_parts(data.0[r].1, None, script, region) };
            }
        } else if let Some(s) = script {
            if let Some(r) = region {
                let result = data
                    .3
                    .binary_search_by_key(&(&s.into(), &r.into()), |(key_s, key_r, _)| {
                        (key_s, key_r)
                    })
                    .ok();
                if let Some(r) = result {
                    // safe because all table entries are well formed.
                    return unsafe { get_lang_from_parts(data.3[r].2, None, None, None) };
                }
            }

            let result = data
                .5
                .binary_search_by_key(&(&s.into()), |(key_s, _)| key_s)
                .ok();
            if let Some(r) = result {
                // safe because all table entries are well formed.
                return unsafe { get_lang_from_parts(data.5[r].1, None, None, region) };
            }
        } else if let Some(r) = region {
            let result = data
                .4
                .binary_search_by_key(&(&r.into()), |(key_r, _)| key_r)
                .ok();
            if let Some(r) = result {
                // safe because all table entries are well formed.
                return unsafe { get_lang_from_parts(data.4[r].1, None, None, None) };
            }
        }

        None
    }

    pub fn remove_likely_subtags(
        lang: Option<TinyStr8>,
        script: Option<TinyStr4>,
        region: Option<TinyStr4>,
    ) -> Option<(Option<TinyStr8>, Option<TinyStr4>, Option<TinyStr4>)> {
        // add_likely_subtags returns None when all 3 components are
        // already filled so don't call it in that case.
        let max_langid = if lang.is_some() && script.is_some() && region.is_some() {
            (lang, script, region)
        } else {
            add_likely_subtags(lang, script, region)?
        };

        if let Some(trial) = add_likely_subtags(max_langid.0, None, None) {
            if trial == max_langid {
                return Some((max_langid.0, None, None));
            }
        }

        if max_langid.2.is_some() {
            if let Some(trial) = add_likely_subtags(max_langid.0, None, max_langid.2) {
                if trial == max_langid {
                    return Some((max_langid.0, None, max_langid.2));
                }
            }
        }

        if max_langid.1.is_some() {
            if let Some(trial) = add_likely_subtags(max_langid.0, max_langid.1, None) {
                if trial == max_langid {
                    return Some((max_langid.0, max_langid.1, None));
                }
            }
        }
        None
    }
}