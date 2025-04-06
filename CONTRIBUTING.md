# Contributing

Thanks for your interest in contributing to Keypunch! Most of the information you're looking for can likely be found in Keypunch's [contribution guide](https://welcome.gnome.org/app/Keypunch) on the Welcome to GNOME website. This document contains information specific to Keypunch that cannot be found in said contribution guide.

## Adding a Language

This is the technical procedure for adding a text language to Keypunch. To request a language and help with the non-technical aspects, [create a language request](https://github.com/bragefuglseth/keypunch/issues/new?assignees=&labels=new+language&projects=&template=language_request.yaml&title=%5BLanguage+Request%5D%3A+) instead. To translate the UI, refer to the section above.

1. Add a word list with approximately 200 words to `data/word_lists/{code}.txt`. Replace `{code}` with the code of your language (e.g. `en` or `nb`). If the language has a corresponding list in [Monkeytype's language directory](https://github.com/monkeytypegame/monkeytype/tree/master/frontend/static/languages), you can download that, rename it and run `scripts/json_to_word_list.sh {file_path}` to generate a plain word list. Replace `{file_path}` with a path to the file. You'll need to have `jq` installed for the script to work.
3. Locate and open `src/text_generation.rs`. This is where all language work takes place.
4. Add the language to the `Language` enum at the alphabetical position of its English name. Use its English name as the variant name, and annotate it with the necessary metadata:

   ```rust
   #[strum(message = "{native_name}", to_string = "{code}")]
   LanguageName,
   ```

   Replace `{native_name}` with the native name of the language, and `{code}` with the language code.

5. If the language has punctuation or spacing that deviates from the default (words separated by spaces, punctuation inserted before and after words, and Western Arabic numerals), add it to the match statements of the `simple` and `advanced` functions. Existing language entries can be used as examples.

6. Build the app and test the implementation.
