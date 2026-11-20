/* lesson.js — 教学工作区共享交互脚本
 * 数据驱动的测验与「先回忆再揭晓」组件。课程 HTML 只放数据，不放行为代码。
 * 用法：
 *   <div class="quiz" data-quiz></div>
 *   <script type="application/json" data-quiz-data>{...}</script>
 *   <div class="recall" data-recall>...</div>
 */
(function () {
  "use strict";

  /* 主题切换：跟随系统，手动覆盖写入 localStorage */
  var root = document.documentElement;
  var saved = null;
  try { saved = localStorage.getItem("lesson-theme"); } catch (e) { /* 隐私模式下忽略 */ }
  if (saved === "light" || saved === "dark") root.setAttribute("data-theme", saved);

  var toggle = document.querySelector("[data-theme-toggle]");
  if (toggle) {
    var syncLabel = function () {
      var dark = root.getAttribute("data-theme") === "dark" ||
        (!root.getAttribute("data-theme") && window.matchMedia &&
         window.matchMedia("(prefers-color-scheme: dark)").matches);
      toggle.textContent = dark ? "浅色" : "深色";
    };
    syncLabel();
    toggle.addEventListener("click", function () {
      var dark = root.getAttribute("data-theme") === "dark" ||
        (!root.getAttribute("data-theme") && window.matchMedia &&
         window.matchMedia("(prefers-color-scheme: dark)").matches);
      var next = dark ? "light" : "dark";
      root.setAttribute("data-theme", next);
      try { localStorage.setItem("lesson-theme", next); } catch (e) { /* 忽略 */ }
      syncLabel();
    });
  }

  /* 测验：选项顺序在每种题目数据里已经打乱，脚本不再重排，避免误伤答案位置 */
  var dataScripts = document.querySelectorAll("[data-quiz-data]");
  dataScripts.forEach(function (script) {
    var host = script.previousElementSibling;
    if (!host || !host.hasAttribute("data-quiz")) return;
    var quiz;
    try { quiz = JSON.parse(script.textContent); } catch (e) { return; }

    var answered = 0, correct = 0;
    var scoreEl = document.createElement("div");
    scoreEl.className = "quiz-score";

    quiz.questions.forEach(function (q, qi) {
      var item = document.createElement("div");
      item.className = "quiz-item";

      var num = document.createElement("div");
      num.className = "qnum";
      num.textContent = "第 " + (qi + 1) + " / " + quiz.questions.length + " 题";
      item.appendChild(num);

      var text = document.createElement("div");
      text.className = "qtext";
      text.textContent = q.q;
      item.appendChild(text);

      var opts = document.createElement("div");
      opts.className = "options";
      var feedback = document.createElement("div");
      feedback.className = "feedback";

      q.options.forEach(function (opt, oi) {
        var btn = document.createElement("button");
        btn.type = "button";
        btn.className = "opt";
        btn.textContent = String.fromCharCode(65 + oi) + ". " + opt.t;
        btn.addEventListener("click", function () {
          if (item.dataset.done) return;
          item.dataset.done = "1";
          answered++;
          var ok = !!opt.c;
          if (ok) correct++;
          q.options.forEach(function (other, ai) {
            var b = opts.children[ai];
            b.disabled = true;
            if (other.c) b.classList.add("correct");
          });
          if (!ok) btn.classList.add("wrong");
          feedback.innerHTML = (ok ? "<b>答对了。</b> " : "<b>再看一眼。</b> ") + q.e;
          if (answered === quiz.questions.length) {
            scoreEl.textContent = "本组：" + correct + " / " + answered + " 题答对。";
          }
        });
        opts.appendChild(btn);
      });

      item.appendChild(opts);
      item.appendChild(feedback);
      host.appendChild(item);
    });
    host.appendChild(scoreEl);
  });

  /* 先回忆再揭晓 */
  document.querySelectorAll("[data-recall]").forEach(function (box) {
    var btn = box.querySelector("[data-reveal]");
    var answer = box.querySelector(".answer");
    if (!btn || !answer) return;
    btn.addEventListener("click", function () {
      answer.hidden = !answer.hidden;
      btn.textContent = answer.hidden ? "揭晓参考答案" : "收起参考答案";
    });
  });
})();
