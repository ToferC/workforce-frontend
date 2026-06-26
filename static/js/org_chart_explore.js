// Vertical, box-based org chart explorer.
//
// Reads the server-built tier+team skeleton from the
// <script type="application/json" id="orgchart-data"> payload and renders a
// classic top-down org chart of boxes (org -> tiers -> teams). Team boxes lazily
// load their roles + people (more boxes) from /{lang}/team/{id}/members.json on
// demand, so the expensive person data is only fetched when drilled into.
//
// Capacity is shown as a coloured top border on team boxes (the heatmap), via
// CSS classes that read the GC theme tokens.
(function () {
  "use strict";

  function band(effort) {
    if (!effort || effort <= 0) return "cap-empty";
    if (effort <= 20) return "cap-light";
    if (effort <= 50) return "cap-moderate";
    return "cap-heavy";
  }

  function el(tag, cls, html) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (html != null) n.innerHTML = html;
    return n;
  }

  function esc(s) {
    var d = document.createElement("div");
    d.textContent = s == null ? "" : String(s);
    return d.innerHTML;
  }

  var LANG = "en";
  // Localized labels, populated from the container's data-l-* attributes
  // (rendered through Fluent server-side) so the JS-built boxes stay bilingual.
  var L = {
    people: "people",
    effort: "effort",
    tier: "Tier",
    vacant: "Vacant",
    noRoles: "No roles in this team.",
    expand: "Expand",
    collapse: "Collapse",
    members: "Show roles and people",
  };

  // ── Box builders ──────────────────────────────────────────────────────────
  function box(node) {
    var b = el("div", "oc-node");
    var title = el("div", "oc-node__title");

    if (node.kind === "org") {
      b.classList.add("oc-node--org");
      title.textContent = node.name;
    } else if (node.kind === "tier") {
      b.classList.add("oc-node--tier");
      title.textContent = node.name;
      b.appendChild(title);
      var meta = el("div", "oc-node__meta");
      meta.textContent = L.tier + " " + (node.tierLevel != null ? node.tierLevel : "?");
      b.appendChild(meta);
      return b;
    } else if (node.kind === "team") {
      b.classList.add("oc-node--team", band(node.effort));
      title.innerHTML =
        '<a href="/' + LANG + "/team/" + esc(node.id) + '">' + esc(node.name) + "</a>";
      b.appendChild(title);
      var tmeta = el("div", "oc-node__meta");
      tmeta.textContent =
        (node.headcount || 0) + " " + L.people + " · " + L.effort + " " + (node.effort || 0);
      b.appendChild(tmeta);
      return b;
    }
    b.appendChild(title);
    return b;
  }

  function memberBox(m) {
    var b = el("div", "oc-node oc-node--role" + (m.vacant ? " oc-node--vacant" : ""));
    var title = el("div", "oc-node__title");
    title.innerHTML =
      '<a href="/' + LANG + "/role/" + esc(m.id) + '">' + esc(m.title) + "</a>";
    b.appendChild(title);

    var meta = el("div", "oc-node__meta");
    if (m.vacant) {
      meta.innerHTML = '<span class="badge bg-danger">' + esc(L.vacant) + "</span>";
    } else if (m.person) {
      meta.innerHTML =
        '<a href="/' + LANG + "/person/" + esc(m.person.id) + '">' +
        esc(m.person.name) + "</a> · " + esc(L.effort) + " " + (m.effort || 0);
    }
    b.appendChild(meta);
    return b;
  }

  // ── Tree builders ─────────────────────────────────────────────────────────
  function buildNode(node, depth) {
    var li = el("li");
    var b = box(node);
    li.appendChild(b);

    var kids = node.children || [];
    if (kids.length) {
      var ul = el("ul");
      kids.forEach(function (c) {
        ul.appendChild(buildNode(c, depth + 1));
      });
      li.appendChild(ul);
      // Collapse deeper tiers by default so large orgs stay readable.
      if (depth >= 2) li.classList.add("collapsed");
      b.appendChild(collapseToggle(li));
    }

    if (node.kind === "team") {
      b.appendChild(membersToggle(li, node));
    }
    return li;
  }

  function collapseToggle(li) {
    var t = el("button", "oc-toggle");
    t.type = "button";
    function sync() {
      var collapsed = li.classList.contains("collapsed");
      t.textContent = collapsed ? "+" : "−"; // + / minus
      t.setAttribute("aria-expanded", String(!collapsed));
      t.setAttribute("aria-label", collapsed ? L.expand : L.collapse);
    }
    sync();
    t.addEventListener("click", function (e) {
      e.stopPropagation();
      li.classList.toggle("collapsed");
      sync();
    });
    return t;
  }

  function membersToggle(li, node) {
    var t = el("button", "oc-toggle");
    t.type = "button";
    t.textContent = "+";
    t.setAttribute("aria-label", L.members);
    t.setAttribute("aria-expanded", "false");
    t.addEventListener("click", function (e) {
      e.stopPropagation();
      if (li.dataset.loaded) {
        li.classList.toggle("collapsed");
        var open = !li.classList.contains("collapsed");
        t.textContent = open ? "−" : "+";
        t.setAttribute("aria-expanded", String(open));
        return;
      }
      loadMembers(li, node, t);
    });
    return t;
  }

  function loadMembers(li, node, toggle) {
    toggle.textContent = "…"; // ellipsis while loading
    toggle.disabled = true;
    fetch("/" + LANG + "/team/" + node.id + "/members.json", {
      headers: { Accept: "application/json" },
    })
      .then(function (r) {
        return r.ok ? r.json() : Promise.reject(r.status);
      })
      .then(function (members) {
        var ul = el("ul");
        if (!members || !members.length) {
          var li0 = el("li");
          li0.appendChild(el("div", "oc-node oc-node--role", '<div class="oc-node__meta">' + esc(L.noRoles) + "</div>"));
          ul.appendChild(li0);
        } else {
          members.forEach(function (m) {
            var mli = el("li");
            mli.appendChild(memberBox(m));
            ul.appendChild(mli);
          });
        }
        li.appendChild(ul);
        li.dataset.loaded = "1";
        li.classList.remove("collapsed");
        toggle.textContent = "−";
        toggle.disabled = false;
        toggle.setAttribute("aria-expanded", "true");
      })
      .catch(function () {
        toggle.textContent = "!";
        toggle.disabled = false;
        toggle.setAttribute("aria-label", "Failed to load — click to retry");
        delete li.dataset.loaded;
      });
  }

  // ── Init ──────────────────────────────────────────────────────────────────
  function render() {
    var host = document.getElementById("orgchart");
    var dataEl = document.getElementById("orgchart-data");
    if (!host || !dataEl) return;

    LANG = host.getAttribute("data-lang") || "en";

    var d = host.dataset;
    L = {
      people: d.lPeople || L.people,
      effort: d.lEffort || L.effort,
      tier: d.lTier || L.tier,
      vacant: d.lVacant || L.vacant,
      noRoles: d.lNoroles || L.noRoles,
      expand: d.lExpand || L.expand,
      collapse: d.lCollapse || L.collapse,
      members: d.lMembers || L.members,
    };

    var payload;
    try {
      payload = JSON.parse(dataEl.textContent || "{}");
    } catch (e) {
      return;
    }

    var root = {
      name: host.getAttribute("data-org-name") || "Organization",
      kind: "org",
      children: payload.children || [],
    };

    host.innerHTML = "";
    var wrap = el("div", "orgchart-wrap");
    var chart = el("div", "orgchart");
    var ul = el("ul");
    ul.appendChild(buildNode(root, 0));
    chart.appendChild(ul);
    wrap.appendChild(chart);
    host.appendChild(wrap);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render);
  } else {
    render();
  }
})();
