# Emoji Anki: learn with emojis

[Emoji Anki](https://anisse.github.io/emojianki/) lets you generate [Anki flashcards](https://en.wikipedia.org/wiki/Anki) containing just emojis, as well as their description in 150+ languages. You can use it with your native language to learn flags, or emoji names. Or use it with another language to learn the vocabulary associated with emoji objects, animals & nature, etc.

Select a language, select one ore more Emoji categories, and download the Anki deck; [generation happens directly on the website](https://anisse.github.io/emojianki/).

# How it's built

The emoji data, the language and category names in various languages, all come from the [Unicode Common Locale Data Repository](https://github.com/unicode-org/cldr). It is built in Rust, on top of [genanki-wasm](https://github.com/viniciusdutra314/genanki-wasm/), as well as [quick-xml](https://docs.rs/quick-xml/latest/quick_xml/index.html) and [ICU](https://docs.rs/icu/latest/icu/) for parsing, and the various wasm-bindgen crates. The web UI uses [MVP.css](https://andybrewer.github.io/mvp/) and a [catppuccin](https://catppuccin.com/) color theme.

The website is fully static, with the code doing the parsing and Anki flashcards generation running from the browser. It's hosted on Github from this repository.

# Contributing

The project does not accept unprompted contributions. Please open an issue before sending a PR.

# TODO

It is still early, and there a few things that could be added or fixed.

 - Stop shipping the whole CLDR XML files, but pre-parse only the necessary parts.
 - Test with more implementations than just Anki desktop and Ankidroid
 - A few more customization options
