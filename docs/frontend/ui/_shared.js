(function () {
  var STORAGE_KEY = 'mediavault-theme';

  function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    document.querySelectorAll('[data-theme-toggle]').forEach(function (btn) {
      btn.textContent = theme === 'light' ? '🌙 ダークモードに切替' : '☀️ ライトモードに切替';
    });
  }

  var saved = localStorage.getItem(STORAGE_KEY) || 'dark';
  applyTheme(saved);

  document.addEventListener('click', function (e) {
    var btn = e.target.closest('[data-theme-toggle]');
    if (!btn) return;
    var current = document.documentElement.getAttribute('data-theme') || 'dark';
    var next = current === 'light' ? 'dark' : 'light';
    localStorage.setItem(STORAGE_KEY, next);
    applyTheme(next);
  });
})();

/* お気に入りトグル: PATCH /items/{id} { is_favorite } を想定 */
document.addEventListener('click', function (e) {
  var btn = e.target.closest('[data-favorite-toggle]');
  if (!btn) return;
  btn.classList.toggle('is-active');
});

/* ステータス切替ポップオーバー: 選択で PATCH /items/{id}/status { status } を想定 */
document.addEventListener('click', function (e) {
  var trigger = e.target.closest('[data-status-trigger]');
  var option = e.target.closest('.status-option');

  if (trigger) {
    var popover = trigger.closest('.status-switcher').querySelector('.status-popover');
    popover.hidden = !popover.hidden;
    return;
  }

  if (option) {
    var switcher = option.closest('.status-switcher');
    var t = switcher.querySelector('[data-status-trigger]');
    t.querySelector('.icon').outerHTML = option.querySelector('.icon').outerHTML;
    t.querySelector('.label').textContent = option.querySelector('.label').textContent;
    t.setAttribute('data-current', option.getAttribute('data-status'));
    switcher.querySelector('.status-popover').hidden = true;
    return;
  }

  if (!e.target.closest('.status-switcher')) {
    document.querySelectorAll('.status-popover').forEach(function (p) { p.hidden = true; });
  }
});

/* 評価変更: クリックで即時反映(整数のみ、半星非対応)。PATCH /items/{id} { rating } を想定 */
document.addEventListener('mouseover', function (e) {
  var star = e.target.closest('.star-btn');
  if (!star) return;
  var stars = Array.prototype.slice.call(star.parentElement.querySelectorAll('.star-btn'));
  var index = stars.indexOf(star);
  stars.forEach(function (s, i) {
    s.querySelector('.icon').classList.toggle('is-full', i <= index);
  });
});

document.addEventListener('mouseout', function (e) {
  var container = e.target.closest('.rating-stars');
  if (!container) return;
  var current = parseInt(container.getAttribute('data-rating') || '0', 10);
  var stars = Array.prototype.slice.call(container.querySelectorAll('.star-btn'));
  stars.forEach(function (s, i) {
    s.querySelector('.icon').classList.toggle('is-full', i < current);
  });
});

document.addEventListener('click', function (e) {
  var star = e.target.closest('.star-btn');
  if (!star) return;
  var container = star.closest('.rating-stars');
  var rating = parseInt(star.getAttribute('data-rating'), 10);
  container.setAttribute('data-rating', rating);
  container.querySelector('.val').textContent = rating.toFixed(1);
});

/* タグ/カテゴリ削除: DELETE /items/{id}/tags/{tag_id} または /items/{id}/categories/{category_id} を想定 */
document.addEventListener('click', function (e) {
  var removeBtn = e.target.closest('[data-remove-tag], [data-remove-category]');
  if (!removeBtn) return;
  removeBtn.closest('.tag-pill').remove();
});

/* タグ/カテゴリ追加: POST /items/{id}/tags { name } または /items/{id}/categories { name } を想定 */
document.addEventListener('click', function (e) {
  var addBtn = e.target.closest('[data-tag-add], [data-category-add]');
  if (!addBtn) return;

  var isCategory = addBtn.hasAttribute('data-category-add');
  var input = document.createElement('input');
  input.type = 'text';
  input.className = 'tag-add-input';
  input.placeholder = isCategory ? 'カテゴリ名を入力してEnter' : 'タグ名を入力してEnter';

  addBtn.style.display = 'none';
  addBtn.insertAdjacentElement('beforebegin', input);
  input.focus();

  function cancel() {
    input.remove();
    addBtn.style.display = '';
  }

  input.addEventListener('keydown', function (ev) {
    if (ev.key === 'Escape') {
      cancel();
    } else if (ev.key === 'Enter') {
      var name = input.value.trim();
      if (!name) { cancel(); return; }
      var pill = document.createElement('span');
      pill.className = 'tag-pill';
      var removeAttr = isCategory ? 'data-remove-category' : 'data-remove-tag';
      pill.innerHTML = name + '<button type="button" class="tag-remove" ' + removeAttr + ' aria-label="削除">×</button>';
      addBtn.insertAdjacentElement('beforebegin', pill);
      input.value = '';
      input.focus();
    }
  });

  input.addEventListener('blur', function () {
    setTimeout(cancel, 100);
  });
});
