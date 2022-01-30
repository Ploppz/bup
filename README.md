# Compiling in Ubuntu
```
sudo apt install libclang-dev libsodium-dev libssl-dev libgtk-3-dev pkg-config liblzma-dev build-essential
```
# Request for code review

This is a minimal start of a UI project with `iced`.
The goal is to have a UI to manage backups.

There is a `mod config` holder a `struc Directory`, which is essentially the struct that the UI in its entirety serves to
1) provide an overview of, and
2) gives the possibility to the user to edit.

As a start I define `Directory` to have
1) a _name_
2) several _sources_. A source is a directory where data that is backed up is located
3) several _exclude_ strings - just like in a gitignore, what files from the sources to ignore

The application state/configuration is a `Vec<Directory>`.

I know about the general preference in `iced` and Elm, to separate UI state, and logical state (if I can call the configuration that?). I went all the way with this idea, to make the entire `Directory` struct contain no values related to UI at all: it is as if it was defined without any UI in mind.
The consequence of this is that we have to mirror this data structure when creating iced components.

This is in particular done (so far) in the `Editor`, which is where user can edit values of a single Directory.
Note that we need 4 `Vec`s to hold all the UI state of all the different


```
#[derive(Default)]
pub struct Editor {
    directory: Directory,
    error: Option<String>,

    s_name: text_input::State,
    s_new_source: button::State,
    s_new_exclude: button::State,
    s_save_button: button::State,
    s_cancel_button: button::State,

    s_exclude: Vec<text_input::State>,
    s_delete_exclude_button: Vec<button::State>,

    s_source: Vec<FilePicker>,
    s_delete_source_button: Vec<button::State>,
}
```

**Doubt 1** I wonder if this is the right way to do it,
because I noticed several times during development that it was easy
to create bugs due to forgetting to push to _all_ `Vec`s when user creates a new source or exclude.
(and removing from `Vec`s when deleting source/exclude).
It just doesn't seem like a good programming model when you _have_ to remember to do the same operation to
`N` `Vec`s..?

Secondly, not a big problem, but there's a lot of 'zipping' of iterators happening.

All doubts considered, I still like the idea that the `Directory` struct is so separated from the UI.
The `Editor` is one of two "slides" (as in the iced tour), the other one being the overview, which iterates over the Directories showing only the name, and possibility to click it to edit it.


Apart from that, there are many small questions.
- in path.rs I have a folder selector; not sure if I did it the best way there with message passing.
- I have implemented verification in the Editor, of all the values, so that the user gets an error message, like "Name should not be empty" if they try to save. Is this implemented in a good way? It uses the `verify_directory` function in two places separately: 1) in `Ui` to decide whether or not to leave the Editor scene and go back to the overview upon pressing Save, and 2) in `Editor` just to show the eventual error message when user presses Save.
