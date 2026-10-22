// Double-submit guard for post forms. Once a form actually submits (the
// browser and the GCDS components have validated it, and nothing cancelled
// the event), its submit buttons are disabled so a second click can't create
// a duplicate record. They come back when the page is shown again from the
// back/forward cache, and after an HTMX request settles (HTMX forms stay on
// the page).
(function () {
    function buttons(form) {
        return form.querySelectorAll('button[type="submit"], input[type="submit"]');
    }

    function setBusy(form, busy) {
        buttons(form).forEach(function (button) { button.disabled = busy; });
        if (busy) { form.setAttribute('aria-busy', 'true'); } else { form.removeAttribute('aria-busy'); }
    }

    document.addEventListener('submit', function (event) {
        var form = event.target;
        if (event.defaultPrevented || (form.method || '').toLowerCase() !== 'post') { return; }
        // Disable after this tick so the clicked button's own value is still submitted.
        setTimeout(function () { setBusy(form, true); }, 0);
    });

    document.addEventListener('htmx:afterRequest', function (event) {
        var form = event.target.closest && event.target.closest('form');
        if (form) { setBusy(form, false); }
    });

    window.addEventListener('pageshow', function (event) {
        if (event.persisted) {
            document.querySelectorAll('form[aria-busy="true"]').forEach(function (form) { setBusy(form, false); });
        }
    });
})();

// Filter boxes for long selects (forms::filter_select): typing narrows the
// options to those containing every typed word; the current choice and the
// prompt always stay visible.
(function () {
    function setUp(input) {
        var select = document.getElementById(input.getAttribute('data-filter-for'));
        if (!select) { return; }
        input.hidden = false;
        input.addEventListener('input', function () {
            var words = input.value.toLowerCase().split(/\s+/).filter(Boolean);
            Array.prototype.forEach.call(select.options, function (option) {
                var text = option.textContent.toLowerCase();
                var keep = option.value === '' || option.selected || words.every(function (w) { return text.indexOf(w) !== -1; });
                option.hidden = !keep;
            });
        });
    }
    function init(root) {
        (root || document).querySelectorAll('input[data-filter-for]').forEach(setUp);
    }
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () { init(); });
    } else {
        init();
    }
    document.addEventListener('htmx:load', function (event) { init(event.target); });
})();
