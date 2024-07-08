# Contributing

Thanks for your interest in contributing to Keypunch! Here are some useful things to know:

## Issues and suggestions

Bug reports and feature requests are welcome. However, please keep the following in mind:

- Flatpak is the only officially supported packaging format
- The app has a [conservative approach to preferences](https://ometer.com/preferences.html); things should ideally be good enough out of the box
- Before making a change and opening a pull request, make sure to discuss it in the issue tracker first, to make sure that the change will be accepted
- This project follows the [GNOME Code of Conduct](https://conduct.gnome.org)

## Building

Use the Flatpak version of [GNOME Builder](https://apps.gnome.org/Builder) to build and run the project from source: 

1. Open Builder and press the "Clone Repository…" button
2. Paste a link to the repository in the "Repository URL" field:

   ```
   https://github.com/bragefuglseth/keypunch
   ```

3. Press the "Clone repository…" button
4. Press confirm if asked aboout automatic installation of any dependencies, and wait for them to download
5. Press the play button in the header bar

## Translating

Keypunch does not have any external translation infrastructure as of now, but a slot on Weblate will be applied for as soon as the project meets the requirements. Until then, please submit translations as regular pull requests. Translation work happens in the `po` directory.

## Adding a Language

This is the technical procedure for adding a text language to Keypunch. To request a language and help with the non-technical aspects, [create a langauge request](https://github.com/bragefuglseth/keypunch/issues/new?assignees=&labels=new+language&projects=&template=language_request.yaml&title=%5BLanguage+Request%5D%3A+) instead. To translate the UI, refer to the section above.

1. Add a word list with approximately 200 words to `data/word_lists/{code}.txt`. Replace `{code}` with the code of your language (e.g. `en_US` or `nb_NO`). If the language has a corresponding list in [Monkeytype's language directory](https://github.com/monkeytypegame/monkeytype/tree/master/frontend/static/languages), you can download that, rename it and run `scripts/json_to_word_list.sh {file_path}` to generate a plain word list. Replace `{file_path}` with a path to the file. You'll need to have `jq` installed for the script to work.
3. Locate and open `src/text_generation.rs`. This is where all language work takes place.
4. Add the language to the `Language` enum at the alphabetical position of its English name. Use its English name as the variant name, and annotate it with the necessary metadata:

   ```rust
   #[strum(message = "{native_name}", to_string = "{code}")]
   LanguageName,
   ```

   Replace `{native_name}` with the native name of the language, and `{code}` with the language code.

5. Add the language to the match statements of the `simple` and `advanced` functions. If the language can use the "generic" implementation of any of those text types (words separated by spaces, punctuation inserted before and after words, and Western Arabic numerals), you can simply add it to the long, alphabetically sorted `Language | Language | Language | […]` pattern at the beginning. If it can use the generic advanced implementation, but has some special punctuation marks or an alternate numeral system, add it below and imitate the other languages already implemented there.

6. Build the app and test the implementation.
