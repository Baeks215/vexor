(function () {
  function vexor(hljs) {
    return {
      name: "Vexor",
      keywords: {
        keyword: "set val export each fn where match if else",
        literal: "true false Nil PI",
        built_in:
          "Circle Rect Text Group Line Curve Path sample close jumpTo lineTo " +
          "curveTo move scale rotate mirrorX mirrorY fill stroke rgb rgba hsl hsla " +
          "rad deg sin cos tan sinh cosh tanh asinh acosh atanh asin acos atan atan2 " +
          "round floor ceil abs log exp max min clamp magnitude normalize dot " +
          "map filter drop take dropWhile takeWhile foldl foldr zip zipWith flatMap " +
          "enumerate len reverse find sort sortBy repeat fst snd",
      },
      contains: [
        hljs.COMMENT("--", "$"),
        hljs.QUOTE_STRING_MODE,
        hljs.C_NUMBER_MODE,
      ],
    };
  }

  function apply() {
    if (!window.hljs) return;
    hljs.registerLanguage("vexor", vexor);
    // mdBook bundles an old highlight.js: use highlightBlock, fall back to v11 API.
    var run = hljs.highlightElement
      ? function (el) { hljs.highlightElement(el); }
      : function (el) { hljs.highlightBlock(el); };
    document.querySelectorAll("code.language-vexor").forEach(function (el) {
      if (el.dataset) delete el.dataset.highlighted; // clear hljs v11 guard if present
      el.className = "language-vexor"; // reset prior highlight classes
      el.textContent = el.textContent; // drop old token spans
      run(el);
    });
  }

  window.addEventListener("load", apply); // runs after mdBook's own highlight pass
})();
