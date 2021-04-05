===============================================================================
Cool hacks for development
===============================================================================

These represent the views of the authors and are not officially supported
by the Whisperfish project. Corrections, additions and suggestions welcome!

-------------------------------------------------------------------------------
Rust protips
-------------------------------------------------------------------------------

If you want to build locally against local changes to
`<service> https://github.com/Michael-F-Bryan/libsignal-service-rs`_
or `<protocol> https://github.com/Michael-F-Bryan/libsignal-protocol-rs`_
you should edit ``Cargo.toml`` where applicable::

        [patch."https://github.com/Michael-F-Bryan/libsignal-service-rs"]
        libsignal-service = { path = "/PATH/TO/GIT/CLONE/libsignal-service-rs/libsignal-service" }
        libsignal-service-actix = { path = "PATH/TO/GIT/CLONE/libsignal-service-rs/libsignal-service-actix" }

        [patch."https://github.com/Michael-F-Bryan/libsignal-protocol-rs"]
        libsignal-protocol = { path = "/PATH/TO/GIT/CLONE/libsignal-protocol-rs/libsignal-protocol" }

-------------------------------------------------------------------------------
ZSH
-------------------------------------------------------------------------------

Using `autoenv <https://github.com/zpm-zsh/autoenv>`_ you can activate
the environment by symlinking it::

        $ ln -s .env .in

Then you can have an ``.out`` file as well, which is useful for cleaning the
mess from your environment when leaving the directory, or sourcing it manually
to run ``cargo test`` outside the ARM-cross-compilation situation.

Furthermore if you have Python's virtualenv support in your prompt,
you can set its environment variable. A bit of a hacky overload but
having these things visible in the prompt is useful.

.. code:: bash

        unset RUST_SRC_PATH
        unset RUST_BACKTRACE
        unset MERSDK
        unset MER_TARGET
        unset RUSTFLAGS
        unset SSH_TARGET

        test ! -z $VIRTUAL_ENV && unset VIRTUAL_ENV

-------------------------------------------------------------------------------
(Neo)Vim
-------------------------------------------------------------------------------

In the Vim development process, the code is represented by two separate yet
equally important plugins: The autocompleter, which helps with crate contents,
and the linter, which checks for your mistakes. These are their stories. **DUN DUN**.

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
Deoplete
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

`Deoplete <https://github.com/Shougo/deoplete.nvim>`_ is the
preferred completion framework, as it allows for different
sources to be used.

`Deoplete-Rust <https://github.com/sebastianmarkow/deoplete-rust>`_
is the source plugin of choice. It uses `Racer <https://github.com/racer-rust/racer>`_ to do the heavy lifting.

Deoplete-Rust respects the `RUST_SRC_PATH` variable, so all you have to
configure is

.. code:: vim

        let g:deoplete#sources#rust#racer_binary='/path/to/racer'

Note that Racer must be installed from Nightly but it
professes to know all the channels. Have the stable Rust source
available::

        $ rustup component add rust-src --toolchain stable

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
ALE
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

`ALE <https://github.com/dense-analysis/ale>`_ is the preferred
linter, at least for now. It also provides a plugin system, which is suitable
for our needs.

It takes its cues from `rust-analyzer <https://rust-analyzer.github.io/manual.html#rust-analyzer-language-server-binary>`_.

Then configure `g:ale_linters` to include `analyzer`

.. code:: vim

        let g:ale_linters = {
                \ 'rust': ['analyzer']
        }


Note that you may get a ton of `rustc` processes from this approach.

.. code:: vim

        let g:ale_lint_on_save=0

should prevent that from happening. You may see a ton of compilation
happen when starting to edit and running the first `cargo test`, but
after that it should cool down.

-------------------------------------------------------------------------------
Sailfish IDE (QtCreator)
-------------------------------------------------------------------------------

You can setup the Sailfish IDE for working on the UI part. Create a basic `pro`
file and open it as new project in QtCreator:

.. code::
    harbour-whisperfish.pro:
    TARGET = harbour-whisperfish
    CONFIG += sailfishapp_qml

You will be asked to configure "kits". Select only the ARM one and deselect all
build configs except for "debug". (It doesn't matter which, just keep only one.)
Select "project" in the sidebar, choose "build" and click the tiny crossed
circles for all build steps to disable them. This makes sure you don't
accidentally start the build engine (which you don't need).

All QML and JS files will be picked up automatically. Rust files etc. won't show
up, but there are better tools for that anyways.

Then select the "execution" configuration. Disable the "Rsync: Deploys with rsync."
step but keep the "Prepare Target" step. Click "add" below "execution" and
choose "custom executable". Enter in the "executable" field:

.. code::
    path/to/whisperfish/live-patch.sh

Set `-w -B` as command line arguments. This enables watching for changes
(deploying and restarting as needed) and disables automatically rebuilding the
app. (Use `-b` to enable building.)

Now you can click on "run" (or press Ctrl+R) to start the live runner. All log
output will be in the "program output" pane (Alt+3). QML errors will become
clickable links to the respective files.
