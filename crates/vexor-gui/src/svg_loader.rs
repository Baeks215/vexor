//! Custom egui SVG image loader that configures the default and generic font families to fonts
//! present in the loaded system db. Otherwise behaves like `egui_extras`' built-in SVG loader
//! (caching per URI and size).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use eframe::egui;
use egui::ColorImage;
use egui::load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint};
use egui::mutex::Mutex;
use resvg::usvg;

struct Entry {
    last_used: AtomicU64,
    result: Result<Arc<ColorImage>, String>,
}

pub struct VexorSvgLoader {
    pass_index: AtomicU64,
    cache: Mutex<std::collections::HashMap<String, std::collections::HashMap<SizeHint, Entry>>>,
    options: usvg::Options<'static>,
}

impl VexorSvgLoader {
    pub const ID: &'static str = egui::generate_loader_id!(VexorSvgLoader);

    pub fn new() -> Self {
        let mut options = usvg::Options::default();
        options.fontdb_mut().load_system_fonts();

        // Point the default + generic families at fonts that actually exist, preferring common
        // cross-distro families and falling back to the first available face.
        let db = options.fontdb_mut();
        let pick = |prefer: &[&str]| -> Option<String> {
            prefer
                .iter()
                .find(|n| {
                    db.faces().any(|f| {
                        f.families
                            .iter()
                            .any(|(fam, _)| fam.eq_ignore_ascii_case(n))
                    })
                })
                .map(|n| n.to_string())
                .or_else(|| {
                    db.faces()
                        .next()
                        .and_then(|f| f.families.first().map(|(n, _)| n.clone()))
                })
        };
        let sans = pick(&["DejaVu Sans", "Noto Sans", "Liberation Sans", "Arial"]);
        let serif = pick(&[
            "DejaVu Serif",
            "Noto Serif",
            "Liberation Serif",
            "Times New Roman",
        ]);
        let mono = pick(&[
            "DejaVu Sans Mono",
            "Noto Sans Mono",
            "Liberation Mono",
            "Courier New",
        ]);

        if let Some(s) = &sans {
            db.set_sans_serif_family(s.clone());
        }
        if let Some(s) = serif {
            db.set_serif_family(s);
        }
        if let Some(s) = mono {
            db.set_monospace_family(s);
        }
        if let Some(s) = sans {
            options.font_family = s;
        }

        Self {
            pass_index: AtomicU64::new(0),
            cache: Mutex::new(std::collections::HashMap::new()),
            options,
        }
    }
}

fn is_supported(uri: &str) -> bool {
    uri.ends_with(".svg")
}

impl ImageLoader for VexorSvgLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &egui::Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if !is_supported(uri) {
            return Err(LoadError::NotSupported);
        }

        let mut cache = self.cache.lock();
        let bucket = cache.entry(uri.to_owned()).or_default();

        if let Some(entry) = bucket.get(&size_hint) {
            entry
                .last_used
                .store(self.pass_index.load(Relaxed), Relaxed);
            return match entry.result.clone() {
                Ok(image) => Ok(ImagePoll::Ready { image }),
                Err(err) => Err(LoadError::Loading(err)),
            };
        }

        match ctx.try_load_bytes(uri) {
            Ok(BytesPoll::Ready { bytes, .. }) => {
                let result =
                    egui_extras::image::load_svg_bytes_with_size(&bytes, size_hint, &self.options)
                        .map(Arc::new);
                bucket.insert(
                    size_hint,
                    Entry {
                        last_used: AtomicU64::new(self.pass_index.load(Relaxed)),
                        result: result.clone(),
                    },
                );
                match result {
                    Ok(image) => Ok(ImagePoll::Ready { image }),
                    Err(err) => Err(LoadError::Loading(err)),
                }
            }
            Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
            Err(err) => Err(err),
        }
    }

    fn forget(&self, uri: &str) {
        self.cache.lock().retain(|key, _| key != uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .values()
            .flat_map(|bucket| bucket.values())
            .map(|entry| match &entry.result {
                Ok(image) => image.pixels.len() * std::mem::size_of::<egui::Color32>(),
                Err(err) => err.len(),
            })
            .sum()
    }

    fn end_pass(&self, pass_index: u64) {
        self.pass_index.store(pass_index, Relaxed);
        let mut cache = self.cache.lock();
        cache.retain(|_key, bucket| {
            if 2 <= bucket.len() {
                bucket.retain(|_, e| pass_index <= e.last_used.load(Relaxed) + 1);
            }
            !bucket.is_empty()
        });
    }
}
