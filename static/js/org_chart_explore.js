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

  function render() {
    var el = document.getElementById("orgchart");
    if (!el || typeof echarts === "undefined") return;

    var root = buildRoot();
    if (!root) return;

    if (chart) {
      chart.dispose();
    }

    var isDark = document.documentElement.getAttribute("data-theme") !== "light";
    chart = echarts.init(el, isDark ? "dark" : null, { renderer: "canvas" });

    chart.setOption({
      tooltip: {
        trigger: "item",
        triggerOn: "mousemove",
        formatter: function (p) {
          var d = p.data || {};
          if (d.kind === "team") return "Team: " + d.name;
          if (d.kind === "tier") return d.name + " (tier L" + (d.tierLevel != null ? d.tierLevel : "?") + ")";
          return d.name;
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
          },
          leaves: {
            label: { position: "right", verticalAlign: "middle", align: "left" },
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
