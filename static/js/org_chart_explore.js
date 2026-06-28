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

  // A domain-coloured capability chip (reuses the .domain-chip / .domain-* CSS).
  function chip(label, group) {
    var s = el("span", "badge domain-chip domain-" + (group || "corp"));
    s.textContent = label;
    return s;
  }

  function capsRow(items) {
    var row = el("div", "oc-node__caps");
    items.forEach(function (c) {
      if (c && c.label) row.appendChild(chip(c.label, c.group));
    });
    return row;
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
    leadership: "Leadership team",
    workingTier: "Working teams",
    zoomIn: "Zoom in",
    zoomOut: "Zoom out",
    zoomReset: "Reset zoom",
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
      // Leadership tiers (L0–L3) fold their leadership team into this box;
      // working tiers (L4) hold working teams as their own boxes below.
      var leadership = node.leadership;
      b.classList.add(leadership ? "oc-node--leadership" : "oc-node--working");
      // A leadership box doubles as its leadership team, so colour it by the
      // team's capacity band like a team box.
      if (leadership && node.mergedTeams && node.mergedTeams.length) {
        b.classList.add(band(node.effort));
      }
      title.textContent = node.name;
      b.appendChild(title);

      var meta = el("div", "oc-node__meta");
      var lvl = node.tierLevel != null ? node.tierLevel : "?";
      var kindLabel = leadership ? L.leadership : L.workingTier;
      meta.innerHTML =
        '<span class="oc-level">' + L.tier + " " + esc(lvl) + "</span> " +
        '<span class="oc-kind">' + esc(kindLabel) + "</span>";
      // For a merged leadership box, surface the folded-in team's headcount.
      if (leadership && node.mergedTeams && node.mergedTeams.length) {
        meta.innerHTML +=
          " · " + (node.headcount || 0) + " " + esc(L.people) +
          " · " + esc(L.effort) + " " + (node.effort || 0);
      }
      b.appendChild(meta);

      if (node.primaryLabel) {
        b.appendChild(capsRow([{ label: node.primaryLabel, group: node.primaryGroup }]));
      }
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
    if (!m.vacant && m.person && m.person.capabilities && m.person.capabilities.length) {
      b.appendChild(capsRow(m.person.capabilities));
    }
    return b;
  }

  // The teams whose members a node lazily loads: a working team is itself; a
  // leadership tier loads the team(s) merged into its box. Everything else has
  // no lazy members (its structure is in the eager skeleton).
  function teamIdsFor(node) {
    if (node.kind === "team") return [node.id];
    if (node.kind === "tier" && node.mergedTeams && node.mergedTeams.length) {
      return node.mergedTeams.map(function (t) { return t.id; });
    }
    return [];
  }

  // ── Tree builders ─────────────────────────────────────────────────────────
  function buildNode(node, depth) {
    var li = el("li");
    var b = box(node);
    li.appendChild(b);

    // Eager skeleton children (sub-tiers and working teams).
    var kids = node.children || [];
    var ul = null;
    if (kids.length) {
      ul = el("ul");
      kids.forEach(function (c) {
        ul.appendChild(buildNode(c, depth + 1));
      });
      li.appendChild(ul);
    }

    var teamIds = teamIdsFor(node);
    var hasMembers = teamIds.length > 0;

    if (kids.length || hasMembers) {
      // Collapse deeper levels by default so large orgs stay readable.
      if (depth >= 2) li.classList.add("collapsed");
      b.appendChild(nodeToggle(li, ul, teamIds, hasMembers));
    }
    return li;
  }

  // A simple collapse toggle for an already-built subtree (used for member
  // sub-hierarchies under a role).
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

  // The single expand/collapse toggle for a skeleton node. It reveals the
  // eager children and, on first open, lazily loads the node's team members
  // (roles + people, nested by their reporting lines) into the same subtree.
  function nodeToggle(li, ul, teamIds, hasMembers) {
    var t = el("button", "oc-toggle");
    t.type = "button";
    function sync() {
      var collapsed = li.classList.contains("collapsed");
      t.textContent = collapsed ? "+" : "−";
      t.setAttribute("aria-expanded", String(!collapsed));
      t.setAttribute("aria-label", hasMembers ? L.members : (collapsed ? L.expand : L.collapse));
    }
    sync();
    t.addEventListener("click", function (e) {
      e.stopPropagation();
      var willOpen = li.classList.contains("collapsed");
      if (willOpen && hasMembers && !li.dataset.membersLoaded) {
        loadMembers(li, ul, teamIds, t, sync);
        return;
      }
      li.classList.toggle("collapsed");
      sync();
    });
    return t;
  }

  // Append member boxes (and their nested direct reports) to a <ul>.
  function appendMembers(ul, members) {
    members.forEach(function (m) {
      var mli = el("li");
      mli.appendChild(memberBox(m));
      var kids = m.children || [];
      if (kids.length) {
        var sub = el("ul");
        appendMembers(sub, kids);
        mli.appendChild(sub);
        mli.appendChild(collapseToggle(mli));
      }
      ul.appendChild(mli);
    });
  }

  function loadMembers(li, ul, teamIds, toggle, sync) {
    toggle.textContent = "…"; // ellipsis while loading
    toggle.disabled = true;
    Promise.all(teamIds.map(function (id) {
      return fetch("/" + LANG + "/team/" + id + "/members.json", {
        headers: { Accept: "application/json" },
      }).then(function (r) {
        return r.ok ? r.json() : Promise.reject(r.status);
      });
    }))
      .then(function (lists) {
        var members = [];
        lists.forEach(function (l) { if (l && l.length) members = members.concat(l); });
        if (!ul) {
          ul = el("ul");
          li.appendChild(ul);
        }
        if (!members.length) {
          var li0 = el("li");
          li0.appendChild(el("div", "oc-node oc-node--role", '<div class="oc-node__meta">' + esc(L.noRoles) + "</div>"));
          ul.appendChild(li0);
        } else {
          appendMembers(ul, members);
        }
        li.dataset.membersLoaded = "1";
        li.classList.remove("collapsed");
        toggle.disabled = false;
        sync();
      })
      .catch(function () {
        toggle.textContent = "!";
        toggle.disabled = false;
        toggle.setAttribute("aria-label", "Failed to load — click to retry");
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
      leadership: d.lLeadership || L.leadership,
      workingTier: d.lWorkingtier || L.workingTier,
      zoomIn: d.lZoomin || L.zoomIn,
      zoomOut: d.lZoomout || L.zoomOut,
      zoomReset: d.lZoomreset || L.zoomReset,
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
    host.classList.add("oc-host");
    var wrap = el("div", "orgchart-wrap");
    var chart = el("div", "orgchart");
    var ul = el("ul");
    ul.appendChild(buildNode(root, 0));
    chart.appendChild(ul);
    wrap.appendChild(chart);
    host.appendChild(wrap);
    host.appendChild(zoomControls(chart, wrap));
  }

  // ── Zoom ────────────────────────────────────────────────────────────────
  function zoomControls(chart, wrap) {
    var zoom = 1;
    var MIN = 0.4,
      MAX = 2;

    function apply() {
      chart.style.transform = "scale(" + zoom + ")";
    }
    function set(z) {
      zoom = Math.min(MAX, Math.max(MIN, Math.round(z * 10) / 10));
      apply();
    }

    function btn(symbol, label, fn) {
      var b = el("button", null, symbol);
      b.type = "button";
      b.setAttribute("aria-label", label);
      b.title = label;
      b.addEventListener("click", fn);
      return b;
    }

    var box = el("div", "oc-zoom");
    box.appendChild(btn("+", L.zoomIn, function () { set(zoom + 0.1); }));
    box.appendChild(btn("−", L.zoomOut, function () { set(zoom - 0.1); }));
    box.appendChild(btn("⤢", L.zoomReset, function () { set(1); }));

    // Ctrl/⌘ + wheel to zoom over the chart.
    wrap.addEventListener("wheel", function (e) {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        set(zoom + (e.deltaY < 0 ? 0.1 : -0.1));
      }
    }, { passive: false });

    return box;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", render);
  } else {
    render();
  }
})();
