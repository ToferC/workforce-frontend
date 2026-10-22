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
