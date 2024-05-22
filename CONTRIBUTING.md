# Contributing

Thanks for your interest in contributing to Keypunch! Here are some useful 
things to know:

## Building

Use [GNOME Builder][1] to build and run the 
project from source: 

1. Open Builder and press the "Clone Repository…" button
2. Paste a link to the repository in the "Repository URL" field:

   ```
   https://github.com/bragefuglseth/keypunch
   ```

3. Press the "Clone repository…" button
4. Confirm automatic installation of any dependencies and wait for them to 
   download
5. Press the play button in the header bar

## Adding a Language

This is the technical procedure for adding a text language to Keypunch. To
request a language and help with the non-technical aspects, 
[create a langauge request][2] instead.

1. Add a word list with approximately 200 words to `data/word_lists/{code}.txt`.
   Replace `{code}` with the code of your language (e.g. `en_US` or `nb_NO`). If
   the language has a corresponding list in [Monkeytype's language directory][3],
   you can download that, rename it and run `scripts/json_to_word_list.sh` to
   generate a plain word list.

2. Locate and open `src/text_generation.rs`. This is where all language work
   takes place.

3. Add the language to the `Language` enum at the alphabetical position
   of its English name. Use its English name as the variant name, and annotate 
   it with the necessary metadata:

   ```rust
   #[strum(message = "{native_name}", to_string = "{code}")]
   LanguageName,
   ```

   Replace `{native_name}` with the native name of the language, and `{code}` 
   with the language code.

4. Add the language to the match statements of the `simple` and `advanced`
   functions. If the language can use the "generic" implementation of any of
   those text types (words separated by spaces, punctuation inserted before
   and after words, etc., you can simply add it to the long, alphabetically sorted
   `Language | Language | Language | […]` pattern at the beginning. If it can
   use the generic advanced implementation, but has some special punctuation
   marks, add it below and imitate the other languages already implemented there.

5. Build the app and test the implementation.

[1]: https://apps.gnome.org/nb/Builder/
[2]: https://github.com/bragefuglseth/keypunch/issues/new?assignees=&labels=new+language&projects=&template=language_request.yaml&title=%5BLanguage+Request%5D%3A+
[3]: https://github.com/monkeytypegame/monkeytype/tree/master/frontend/static/languages
