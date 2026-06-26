// Visual org chart explorer (Phase 1).
//
// Reads the server-built tier+team skeleton from the <script type="application/json"
// id="orgchart-data"> payload and renders it as an ECharts `tree`. The
// organization name is taken from the container's data-org-name attribute and
// used as a synthetic root so the tree reads org -> tiers -> teams.
//
// ECharts is loaded globally in base.html. Later phases will lazy-load team
// stats (capacity heatmap) and a detail panel; this phase is structure only.
(function () {
  "use strict";

  var chart;

  function buildRoot() {
    var dataEl = document.getElementById("orgchart-data");
    var el = document.getElementById("orgchart");
    if (!dataEl || !el) return null;

    var payload;
    try {
      payload = JSON.parse(dataEl.textContent || "{}");
    } catch (e) {
      return null;
    }

    return {
      name: el.getAttribute("data-org-name") || "Organization",
      kind: "org",
      // The root sits at the far left, so put its label on the right (internal
      // nodes label to the left by default) to avoid clipping the org name.
      label: { position: "right", distance: 9, fontWeight: "bold", align: "left" },
      children: payload.children || [],
    };
  }

  // Capacity heatmap bands, keyed off a team's total active effort. Colors are
  // read from the GC design tokens so they track the light/dark theme.
  function readTokens() {
    var s = getComputedStyle(document.documentElement);
    function tok(name, fallback) {
      var v = s.getPropertyValue(name);
      return (v && v.trim()) || fallback;
    }
    return {
      neutral: tok("--border-color", "#8c8c8c"),
      light: tok("--color-success", "#196636"),
      moderate: tok("--color-warning", "#f2ad0d"),
      heavy: tok("--color-danger", "#b3192e"),
      structural: tok("--color-primary", "#1f497a"),
    };
  }

  function bandColor(effort, t) {
    if (!effort || effort <= 0) return t.neutral; // unstaffed / no load
    if (effort <= 20) return t.light;
    if (effort <= 50) return t.moderate;
    return t.heavy;
  }

  function bandLabel(effort) {
    if (!effort || effort <= 0) return "Empty";
    if (effort <= 20) return "Light";
    if (effort <= 50) return "Moderate";
    return "Heavy";
  }

  // Walk the tree and paint each node: teams by capacity band, tiers/org with
  // the neutral structural color.
  function colorize(node, t) {
    if (node.kind === "team") {
      node.itemStyle = { color: bandColor(node.effort, t) };
      node.symbolSize = 11;
    } else if (!node.itemStyle) {
      node.itemStyle = { color: t.structural };
    }
    (node.children || []).forEach(function (c) {
      colorize(c, t);
    });
  }

  function nodeLabel(p) {
    var d = p.data || {};
    if (d.kind === "team") {
      return d.name + "  ·  " + (d.headcount || 0);
    }
    return d.name;
  }

  function render() {
    var el = document.getElementById("orgchart");
    if (!el || typeof echarts === "undefined") return;

    var root = buildRoot();
    if (!root) return;

    if (chart) {
      chart.dispose();
    }

    var tokens = readTokens();
    colorize(root, tokens);

    var isDark = document.documentElement.getAttribute("data-theme") !== "light";
    chart = echarts.init(el, isDark ? "dark" : null, { renderer: "canvas" });

    chart.setOption({
      tooltip: {
        trigger: "item",
        triggerOn: "mousemove",
        formatter: function (p) {
          var d = p.data || {};
          if (d.kind === "team") {
            return (
              "<strong>" + d.name + "</strong><br/>" +
              (d.headcount || 0) + " people · effort " + (d.effort || 0) +
              " (" + bandLabel(d.effort) + " load)"
            );
          }
          if (d.kind === "tier") {
            return "<strong>" + d.name + "</strong><br/>tier L" + (d.tierLevel != null ? d.tierLevel : "?");
          }
          return "<strong>" + d.name + "</strong>";
        },
      },
      series: [
        {
          type: "tree",
          data: [root],
          // Left-to-right keeps deep hierarchies readable on wide screens.
          orient: "LR",
          top: "2%",
          left: "10%",
          bottom: "2%",
          right: "20%",
          symbol: "circle",
          symbolSize: 9,
          roam: true,
          expandAndCollapse: true,
          // Show org -> tiers -> first level by default; deeper tiers collapse.
          initialTreeDepth: 2,
          label: {
            position: "left",
            verticalAlign: "middle",
            align: "right",
            fontSize: 12,
            formatter: nodeLabel,
          },
          leaves: {
            label: {
              position: "right",
              verticalAlign: "middle",
              align: "left",
              formatter: nodeLabel,
            },
          },
          emphasis: { focus: "descendant" },
          animationDuration: 300,
          animationDurationUpdate: 400,
        },
      ],
    });
  }

  function start() {
    render();
    window.addEventListener("resize", function () {
      if (chart) chart.resize();
    });
    // Re-render so the chart picks up the new light/dark theme.
    document.addEventListener("themechange", render);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", start);
  } else {
    start();
  }
})();
