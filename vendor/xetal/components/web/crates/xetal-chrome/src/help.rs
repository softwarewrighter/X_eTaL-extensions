//! The Help dialog's text: what X_eTaL is, how the live editor works,
//! how to read the code, a short reference and idioms, and links.

use yew::prelude::*;

use crate::footer::REPOSITORY;

fn doc(path: &str) -> String {
    format!("{REPOSITORY}/blob/main/{path}")
}

pub(crate) fn help_text() -> Html {
    html! {
        <>
            <h1>{ "X_eTaL, live" }</h1>
            <p>{ "An experimental, statically typed array language: APL-style \
                  whole-array programming and functional composition, typed as \
                  plain ASCII and drawn typographically. What you type on the left \
                  is drawn decorated on the right as you type." }</p>
            { name() }
            <h2>{ "Editing" }</h2>
            <ul>
                <li>{ "Type in the ASCII pane; the Rendered pane follows, and the \
                       pane below shows each statement's type (or the first error)." }</li>
                <li><b>{ "Run" }</b>{ " (Ctrl-Enter) runs the program and shows its output as it is \
                       printed, with any pictures it draws ([]S_HOW) under it; the classics demos \
                       draw. While it runs, a spinner turns and Run becomes " }<b>{ "Stop" }</b>
                    { "; Clear also stops it." }</li>
                <li><b>{ "Notebook" }</b>{ " runs the whole program showing each statement, drawn, above \
                       its output, as just show does; " }<b>{ "Step" }</b>{ " runs the next statement only (k of n), \
                       the one just run marked; " }<b>{ "Reset" }</b>{ " starts the steps again." }</li>
                <li><b>{ "Boxed" }</b>{ " prints every array framed, as APL2's DISPLAY draws it \
                       (d_isplay gives that picture as a value)." }</li>
                <li><b>{ "Tab" }</b>{ " moves between the panes; the current one has the bright border." }</li>
                <li><b>{ "Zoom" }</b>{ " (Ctrl-.) shows the current pane alone, and back." }</li>
                <li>{ "On a phone the panes stack; Add to Home Screen installs the editor as \
                       an app that works offline." }</li>
                <li>{ "Drag the bars between the panes to resize them (this browser \
                       remembers); double-click a bar to put it back." }</li>
                <li>{ "The drop-down opens a demo, a standard library (shown with its \
                       exports' types) or one of your files; " }<b>{ "Clear" }</b>
                    { " (or \"(empty)\") gives an empty editor to type into." }</li>
                <li><b>{ "Save" }</b>{ " and " }<b>{ "Save as" }</b>{ " keep files in this \
                       browser's local storage. A saved library (lib/Name.xtl) is \
                       imported with u_se<; []N_PUT and []N_GET use the same files \
                       (open work/tttml.model after tttml-train), and []R_EAD asks \
                       for a line. Hello.xtl, a library of your own, is among your \
                       files: the hello-library demo imports it; edit it, Save, \
                       and run the demo again." }</li>
            </ul>
            { reading() }
            { reference() }
            { links() }
        </>
    }
}

/// How the name is said and spelled.
fn name() -> Html {
    html! {
        <>
            <h2>{ "The name" }</h2>
            <p>{ "Say it Ecks-e-tal, as the file type .xtl is said eks-tee-ell. The \
                  logo is an underlined X, a raised e, T, a raised a and L; typed in \
                  ASCII it is X_eTaL (or X_ e:T a:L, which XeTaL itself draws as the \
                  logo), in a sentence XeTaL, and the program is xetal. " }
               <a href={doc("docs/name.md")} target="_blank">{ "Every spelling" }</a>{ "." }</p>
            <img class="name-forms" src="name-forms.png"
                alt="The name every way: said Ecks-e-tal; the logo; the favicon; XeTaL in prose; \
                     X_eTaL typed and drawn; LaTeX; xetal; .xtl; the repository"/>
        </>
    }
}

fn reading() -> Html {
    html! {
        <>
            <h2>{ "Reading the code" }</h2>
            <ul>
                <li>{ "Everything reads right to left, with no precedence: \
                       2 * 3 + 1 is 8." }</li>
                <li>{ "An underlined letter makes a name a function: r_ev is typed \
                       with _ after the r. u: (a raised u) marks your own names." }</li>
                <li>{ "A quote passes a function as a value: '+ r_/ v sums v." }</li>
                <li>{ "Inside { }, _r and _l (drawn as APL's omega and alpha) are the \
                       right and left arguments; x := 5 binds (drawn as an arrow)." }</li>
                <li>{ "A subscript picks an axis: '+ r_/_2 m sums each row of m." }</li>
            </ul>
        </>
    }
}

fn reference() -> Html {
    let rows = [
        ("'+ r_/ v", "reduce: the sum"),
        ("'+ s_\\ v", "scan: running sums"),
        ("'f_ e_ach v", "each item"),
        ("a '* t_able b", "outer product"),
        ("r_ange n", "1 to n"),
        ("2 3 r_eshape v", "reshape"),
        ("t_ally v, s_hape v", "count, shape"),
        ("r_ev v, 1 o_- v", "reverse, rotate"),
        ("i s_elect v", "the items at positions i"),
        ("b r_eplicate v", "keep where b is 1 (or repeat)"),
        ("2 2 2 e_ncode 5, 2 d_ecode v", "radix digits, and back"),
        ("a m_atch b", "same shape and items"),
        ("s_ort v, w_here b", "sort, positions of 1s"),
        ("f_^3 x", "f applied 3 times"),
        ("\"c:\" u_se< \"Combinators\"", "import a library as c:"),
    ];
    html! {
        <>
            <h2>{ "A short reference" }</h2>
            <table>
                { for rows.iter().map(|(code, what)| html! {
                    <tr><td><code>{ *code }</code></td><td>{ *what }</td></tr>
                }) }
            </table>
            <p>{ "Idioms: the mean is ['+ r_/ / t_ally] (a fork); the evens of v are \
                  (w_here 0 = v m_od 2) s_elect v; Life is one line, in life.xtl." }</p>
        </>
    }
}

fn links() -> Html {
    let docs = [
        ("README", "README.md"),
        ("Every built-in", "docs/reference.md"),
        ("Idioms beside other languages", "docs/idioms.md"),
        ("How to type it", "docs/input.md"),
        ("The combinators", "docs/birds.md"),
    ];
    html! {
        <>
            <h2>{ "More" }</h2>
            <ul>
                { for docs.iter().map(|(label, path)| html! {
                    <li><a href={doc(path)} target="_blank">{ *label }</a></li>
                }) }
                <li><a href="poster/index.html" target="_blank">{ "Syntax poster: reading XeTaL on one page" }</a></li>
                <li><a href="literate/index.html" target="_blank">{ "Literate documents" }</a></li>
                <li><a href="latex/index.html" target="_blank">{ "Every line in LaTeX" }</a></li>
            </ul>
        </>
    }
}
