{% if options["skill_lang"] == "en" -%}
# RESOURCES.md Format

`RESOURCES.md` is the curated set of trusted sources for this topic. Knowledge for explainers should be drawn from here, not from parametric guesses. Wisdom comes from the communities listed here.

## Structure

```md
# {Topic} Resources

## Knowledge

- [Book: _The Science and Practice of Strength Training_ by Zatsiorsky & Kraemer](https://example.com)
  Foundational text on programming and adaptation. Use for: anything to do with periodisation, recovery, intensity zones.
- [Article: "How Much Should I Train?" by Greg Nuckols (Stronger By Science)](https://example.com)
  Evidence-based review of volume landmarks. Use for: weekly set targets per muscle group.

## Wisdom (Communities)

- [r/weightroom](https://reddit.com/r/weightroom)
  High-signal subreddit, moderated against bro-science. Use for: programme critique, plateau troubleshooting.
- Local: Tuesday strength class at {gym name}
  Use for: real-time coaching feedback on lifts.
```

## Rules

- **High-trust only.** Prefer primary sources, recognised experts, peer-reviewed work, and communities with strong moderation. If a resource is marketing dressed as education, leave it out.
- **Annotate every entry.** A bare link is useless in three months. Add one line: what it covers and when to reach for it.
- **Group by Knowledge / Wisdom.** Mirrors the philosophy in [SKILL.md](./SKILL.md). It is fine for a resource to appear in only one group.
- **Surface gaps explicitly.** If no good resource exists for an area the mission needs, write a `## Gaps` section listing what is missing. This drives future search.
- **Prune ruthlessly.** A resource that turned out to be wrong, shallow, or off-mission should be removed, not buried. Better five sharp sources than thirty mediocre ones.
- **Record community preferences.** If the user has opted out of joining communities, note it here so future sessions don't keep proposing them.
{%- else -%}
# RESOURCES.md 格式规范

`RESOURCES.md` 是本主题经过筛选的可信资料清单。讲解所需的知识应取材于此，而不是依赖模型参数猜测；实践智慧则来自这里列出的社区。

## 结构

```md
# {主题} 资源清单

## 知识（Knowledge）

- [书籍：_The Science and Practice of Strength Training_（Zatsiorsky & Kraemer）](https://example.com)
  训练编排与适应机制的奠基著作。适用场景：周期化安排、恢复与强度区间。
- [文章："How Much Should I Train?"（Greg Nuckols，Stronger By Science）](https://example.com)
  基于证据的训练量边界综述。适用场景：各肌群每周组数目标。

## 智慧（社区 / Wisdom）

- [r/weightroom](https://reddit.com/r/weightroom)
  高信噪比板块，严格清理伪科学。适用场景：训练计划评审、平台期排查。
- 线下：每周二在 {健身房名称} 的力量课
  适用场景：动作实时纠错反馈。
```

## 规则

- **只收高可信度来源。** 优先一手资料、公认专家、同行评审成果以及管理严格的社区。包装成科普的营销内容一律剔除。
- **每条都要加批注。** 裸链接三个月后就毫无用处。加一行说明：它涵盖什么、何时查阅。
- **按「知识 / 智慧」分组。** 呼应 [SKILL.md](./SKILL.md) 的理念。某条资源只出现在其中一组完全正常。
- **显式标出缺口。** 如果使命所需某个领域找不到好资料，写一个 `## 缺口（Gaps）` 小节列出缺失项，驱动后续检索。
- **果断修剪。** 事实证明有误、肤浅或偏离使命的资源应直接删掉，不要留着凑数。五份犀利的资料胜过三十份平庸资料。
- **记录社区偏好。** 如果用户明确表示不想加入社区，在此记录，避免后续会话反复推荐。
{%- endif %}
