use cityhash::cityhash_1_1_1::city_hash_64;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};

use crate::exercise::Exercise;
use crate::render::Render;

pub struct Reveal {
    eqnumber: usize,
}

impl Reveal {
    pub fn new() -> Self {
        Self { eqnumber: 0 }
    }

    fn hash(&self, input: &str) -> String {
        let hash: u64 = city_hash_64(input.as_bytes());

        format!("section-{:x}", hash)
    }

    fn display_eq(&mut self, input: &str) -> String {
        lazy_static! {
            static ref RE_DISPLAY: Regex = Regex::new(
                r"(?x)                 # extended mode
               \[\[\[                  # initial parantheses
               (.+?)                   # 1 definition
               \]\]\]
               "
            )
            .unwrap();
        }
        let mut output = String::new();

        // it is a cursor/offset in line
        let mut it: usize = 0;

        for cap in RE_DISPLAY.captures_iter(input) {
            // pass everythin before interpolation
            let m = cap.get(0).unwrap();
            output += &input[it..m.start()];

            // eq
            output += &format!("  \\[{}\\]\n\n", &cap[1]);
            self.eqnumber += 1;

            // move it
            it = m.end();
        }

        if it < input.len() {
            output += &input[it..];
        }

        output
    }

    fn inline_eq(&mut self, input: &str) -> String {
        lazy_static! {
            static ref RE_DISPLAY: Regex = Regex::new(
                r"(?x)                 # extended mode
               \(\(\(                  # initial parantheses
               (.+?)                   # 1 definition
               \)\)\)
               "
            )
            .unwrap();
        }
        let mut output = String::new();

        // it is a cursor/offset in line
        let mut it: usize = 0;

        for cap in RE_DISPLAY.captures_iter(input) {
            // pass everythin before interpolation
            let m = cap.get(0).unwrap();
            output += &input[it..m.start()];

            // eq
            output += &format!(" \\({}\\) ", &cap[1]);
            self.eqnumber += 1;

            // move it
            it = m.end();
        }

        if it < input.len() {
            output += &input[it..];
        }

        output
    }
}

impl Render for Reveal {
    fn generate_output_filename(&self, output_dir: &Path, input: &Path) -> PathBuf {
        let mut output = output_dir.to_path_buf();
        output.push(input.file_name().unwrap());
        output.set_extension("html");

        output
    }

    fn begin_exercise(&self, exercise: &Exercise) -> String {
        let output = format!(
            r#"<!doctype html>
<html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">

        <title>{}</title>
        <script>var uuid ="{}"</script>

        <link rel="stylesheet" href="dist/reset.css">
        <link rel="stylesheet" href="dist/reveal.css">
        <link rel="stylesheet" href="dist/theme/black.css">

        <!-- Theme used for syntax highlighted code -->
        <link rel="stylesheet" href="plugin/highlight/monokai.css">

        <style>
            .scrollable-slide {{
                height: 700px;
                overflow-y: auto !important;
            }}

            ::-webkit-scrollbar {{
                width: 6px;
            }}

            ::-webkit-scrollbar-track {{
                -webkit-box-shadow: inset 0 0 6px rgba(0,0,0,0.3);
            }}

            ::-webkit-scrollbar-thumb {{
                background-color: #333;
            }}

            ::-webkit-scrollbar-corner {{
                background-color: #333;
            }}
        </style>
        <style>
            .reveal {{
                font-size: 20px;
            }}
            .reveal p {{
                text-align: justify; 
            }}

        </style>
    </head>
    <body>
        <div class="reveal">
            <div class="slides">
"#,
            exercise.title,
            exercise.uuid,
        );

        output
    }

    fn end_exercise(&self, _exercise: &Exercise) -> String {
        String::from(
            r#"
            </div>
        </div>

        <script src="dist/reveal.js"></script>
        <script src="plugin/math/math.js"></script>
        <script>
            Reveal.initialize({
                hash: true,
                progress: false,
                controls: false,
                keyboard: false,
                // Transition style
                transition: 'slide', // none/fade/slide/convex/concave/zoom

                // Transition speed
                transitionSpeed: 'default', // default/fast/slow

                math: {
                    mathjax: 'https://cdn.jsdelivr.net/gh/mathjax/mathjax@2/MathJax.js',
                    config: 'TeX-AMS_HTML-full',
                },

                plugins: [ RevealMath ]
            });
        </script>

        <script>
            // auto scroll
            function resetSlideScrolling(slide) {
                slide.classList.remove('scrollable-slide');
            }

            function handleSlideScrolling(slide) {
                if (slide.scrollHeight >= 700) {
                    slide.classList.add('scrollable-slide');
                }
            }

            Reveal.addEventListener('ready', function (event) {
                handleSlideScrolling(event.currentSlide);
            });

            Reveal.addEventListener('slidechanged', function (event) {
                if (event.previousSlide) {
                    resetSlideScrolling(event.previousSlide);
                }
                handleSlideScrolling(event.currentSlide);
                });
        </script>
    </body>
</html>
"#,
        )
    }

    fn begin_passage(&self, id: &str) -> String {
        format!("\n<section id=\"{}\">\n", self.hash(id))
    }

    fn end_passage(&self, id: &str) -> String {
        format!("<!-- {} --></section>\n", self.hash(id))
    }

    fn text(&mut self, text: &str) -> String {
        let mut output = String::new();
        let mut in_paragraph = false;

        let paragraphs: Vec<&str> = text.split('\n').collect();
        for line in paragraphs {
            let mut line = line.trim().to_string();

            // empty line
            if line.is_empty() {
                if in_paragraph {
                    output += "\n  </p>\n\n";
                    in_paragraph = false;
                }
                continue;
            }

            // normal line
            if !in_paragraph {
                output += "  <p>\n    ";
                in_paragraph = true;
            }
            line = self.display_eq(&line);
            line = self.inline_eq(&line);
            output = output + &line + " ";
        }

        if in_paragraph {
            output += "\n  </p>\n\n";
        }

        output
    }

    fn link(&self, text: &str, target: &str) -> String {
        format!(
            "  <p><a href=\"#/{}\">{}</a></p>\n\n",
            self.hash(target),
            text
        )
    }

    fn begin_choices(&self, text: &str) -> String {
        let mut output = String::new();
        output += "  <hr/>\n\n  <div>";
        output = output + text + "</div>\n\n";

        output
    }

    fn end_choices(&self, _text: &str) -> String {
        let mut output = String::new();
        output += "  <hr/>\n\n <div>";
        output += &format!("  <p style=\"text-align:right\"><a href=\"#/{}\">Volver al inicio</a></p>\n\n",
                           self.hash("Start"));
        output
    }

    fn begin_option(&self, _text: &str, target: &str) -> String {
        format!("  <div> <!-- Option {} -->\n", self.hash(target))
    }

    fn end_option(&self, id: &str) -> String {
        format!("  </div> <!-- EndOption {} -->\n\n", self.hash(id))
    }
}
