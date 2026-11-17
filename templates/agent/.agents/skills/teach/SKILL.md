---
name: teach
description: >-
{%- if options["skill_lang"] == "en" %}
  Teach the user a new skill or concept, within this workspace.
{%- else %}
  在当前工作区内向用户教授一项新技能或新概念。
{%- endif %}
disable-model-invocation: true
argument-hint: "What would you like to learn about?"
---

{% if options["skill_lang"] == "en" -%}
The user has asked you to teach them something. This is a stateful request - they intend to learn the topic over multiple sessions.

## Teaching Workspace

Treat the current directory as a teaching workspace. Workspace paths (`./lessons/` and the rest) resolve from the directory `/teach` was run in; only the `*-FORMAT.md` links resolve from this skill's folder. The state of their learning is captured in this directory in several files:

- `MISSION.md`: A document capturing the _reason_ the user is interested in the topic. This should be used to ground all teaching. Use the format in [MISSION-FORMAT.md](./MISSION-FORMAT.md).
- `./reference/*.html`: A directory of reference materials. These are the compressed learnings from the lessons - cheat sheets, reference algorithms, syntax, yoga poses, glossaries. They are the raw units of learning. They should be beautiful documents which print out well, and are designed for quick reference.
- `RESOURCES.md`: A list of resources which can be explored to ground your teaching in contextual knowledge, or to acquire knowledge and wisdom. Use the format in [RESOURCES-FORMAT.md](./RESOURCES-FORMAT.md).
- `./learning-records/*.md`: A directory of learning records, which capture what the user has learned. These are loosely equivalent to architectural decision records in software development - they capture non-obvious lessons and key insights that may need to be revised later, or drive future sessions. These should be used to calculate the zone of proximal development. They are titled `0001-<dash-case-name>.md`, where the number increments each time. Use the format in [LEARNING-RECORD-FORMAT.md](./LEARNING-RECORD-FORMAT.md).
- `./lessons/*.html`: A directory of lessons. A **lesson** is a single, self-contained HTML output that teaches one tightly-scoped thing tied to the mission. This is the primary unit of teaching in this workspace.
- `./assets/*`: Reusable **components** shared across lessons. See [Assets](#assets).
- `NOTES.md`: A scratchpad for you to jot down user preferences, or working notes.

## Philosophy

To learn at a deep level, the user needs three things:

- **Knowledge**, captured from high-quality, high-trust resources
- **Skills**, acquired through highly-relevant interactive lessons devised by you, based on the knowledge
- **Wisdom**, which comes from interacting with other learners and practitioners

Before the `RESOURCES.md` is well-populated, your focus should be to find high-quality resources which will help the user acquire knowledge. Never trust your parametric knowledge.

Some topics may require more skills than knowledge. Learning more about theoretical physics might be more knowledge-based. For yoga, more skills-based.

### Fluency vs Storage Strength

You should be careful to split between two types of learning:

- **Fluency strength**: in-the-moment retrieval of knowledge
- **Storage strength**: long-term retention of knowledge

Fluency can give the user an illusory sense of mastery, but storage strength is the real goal. Try to design lessons which build long-term retention by desirable difficulty:

- Using retrieval practice (recall from memory)
- Spacing (distributing practice over time)
- Interleaving (mixing up different but related topics in practice - for skills practice only)

## Lessons

A lesson is the main thing you produce: the unit in which knowledge and skills reach the user. Each lesson is one self-contained HTML file, saved to `./lessons/` and titled `0001-<dash-case-name>.html` where the number increments each time.

A lesson should be **beautiful**, with clean, readable typography and layout, since the user will return to these later to review. Think Tufte.

The lesson should be short, and completable very quickly. Learners' working memory is very small, and we need to stay within it. But each lesson should give the user a single tangible win that they can build on. It should be directly tied to the mission, and should be in the user's zone of proximal development.

If possible, open the lesson file for the user by running a CLI command.

Each lesson should link via HTML anchors to other lessons and reference documents.

Each lesson should recommend a primary source for the user to read or watch. This should be the most high-quality, high-trust resource you found on the topic.

Each lesson should contain a reminder to ask followup questions to the agent. The agent is their teacher, and can assist with anything that's unclear.

## Assets

Lessons are built from reusable **components**, stored in `./assets/`: stylesheets, quiz widgets, simulators, diagram helpers, and anything else a second lesson could reuse.

Reuse is the default, not the exception. Before authoring a lesson, read `./assets/` and build from the components already there. When a lesson needs something new and reusable, write it as a component in `./assets/` and link to it; never inline code a future lesson would duplicate.

A shared stylesheet is the first component every workspace earns: every lesson links it, so the lessons look like one consistent course rather than a pile of one-offs. As the workspace grows, so should the component library.

## The Mission

Every lesson should be tied into the mission - the reason that the user is interested in learning about the topic.

If the user is unclear about the mission, or the `MISSION.md` is not populated, your first job should be to question the user on why they want to learn this.

Failing to understand the mission will mean knowledge acquisition is not grounded in real-world goals. Lessons will feel too abstract. You will have no way of judging what the user should do next.

Missions may change as the user develops more skills and knowledge. This is normal - make sure to update the `MISSION.md` and add a learning record to capture the change. Confirm with the user before changing the mission.

## Zone Of Proximal Development

Each lesson, the user should always feel as if they are being challenged 'just enough'.

The user may specify an exact thing they want to learn. If they don't, figure out their zone of proximal development by:

- Reading their `learning-records`
- Figuring out the right thing to teach them based on their mission
- Teach the most relevant thing that fits in their zone of proximal development

## Knowledge

Lessons should be designed around a skill the user is going to learn. The knowledge in the lesson should be only what's required to acquire that skill. You teach the knowledge first, then get the user to practice the skills via an interactive feedback loop.

Knowledge should first be gathered from trusted resources. Use `RESOURCES.md` to keep track of them. Lessons should be littered with citations - links to external resources to back up any claim made. This increases the trustworthiness of the lesson.

For acquiring knowledge, difficulty is the enemy. It eats working memory you need for understanding.

## Skills

If knowledge is all about acquisition, skills are about durability and flexibility. Make the knowledge stick.

For skill acquisition, difficulty is the tool. Effortful retrieval is what builds storage strength. Skills should be taught through interactive lessons. There are several tools at your disposal:

- Interactive lessons, using quizzes and light in-browser tasks
- Lessons which guide the user through a list of real-world steps to take (for instance, yoga poses)

Each of these should be based on a **feedback loop**, where the user receives feedback on their performance. This feedback loop should be as tight as possible, giving feedback immediately - and ideally automatically.

For quizzes, each answer should be exactly the same number of words (and characters, if possible). Vary which position holds the correct answer across questions. Don't give the user any clues about the answer through formatting or order.

## Acquiring Wisdom

Wisdom comes from true real-world interaction - testing your skills outside the learning environment.

When the user asks a question that appears to require wisdom, your default posture should be to attempt to answer - but to ultimately delegate to a **community**.

A community is a place (online or offline) where the user can test their skills in the real world. This might be a forum, a subreddit, a real-world class (budget permitting) or a local interest group.

You should attempt to find high-reputation communities the user can join. If the user expresses a preference that they don't want to join a community, respect it.

## Reference Documents

While creating lessons, you should also create reference documents. Lessons can reference these documents - they are useful for tracking raw units of knowledge useful across lessons.

Lessons will rarely be revisited later - reference documents will be. They should be the compressed essence of the lesson, in a format designed for quick reference.

Some learning topics lend themselves to reference:

- Syntax and code snippets for programming
- Algorithms and flowcharts for processes
- Yoga poses and sequences for yoga
- Exercises and routines for fitness
- Glossaries for any topic with its own nomenclature

Glossaries, in particular, are an essential reference. Once one is created, it should be adhered to in every lesson.

## `NOTES.md`

The user will sometimes express preferences of how they want to be taught, or things you should keep in mind. This is the place to record those preferences, so you can refer back to them when designing lessons or working with the user.
{%- else -%}
用户要求你教他某项内容。这是一个有状态的请求——他打算通过多次会话来系统学习该主题。

## 教学工作区

将当前目录视为一个教学工作区。工作区路径（`./lessons/` 等）以运行 `/teach` 时的目录为基准解析；只有 `*-FORMAT.md` 链接相对于本技能所在目录解析。用户的学习状态记录在当前目录的以下文件中：

- `MISSION.md`：记录用户对该主题感兴趣的*原因*，用于锚定全部教学决策。格式见 [MISSION-FORMAT.md](./MISSION-FORMAT.md)。
- `./reference/*.html`：参考资料目录，存放从课程中提炼出的精华——速查表、参考算法、语法、瑜伽体式、术语表等。它们是学习的原始知识单元，应排版精美、适合打印且便于快速查阅。
- `RESOURCES.md`：可探索的资源清单，用于为讲解提供有据可依的知识背景，或获取知识与智慧。格式见 [RESOURCES-FORMAT.md](./RESOURCES-FORMAT.md)。
- `./learning-records/*.md`：学习记录目录，记录用户已经掌握的内容。大致相当于软件开发中的架构决策记录（ADR）——记录非显而易见的收获与关键洞见，供后续修订或驱动后续会话，并用于计算最近发展区。命名为 `0001-<dash-case-name>.md`，编号依次递增。格式见 [LEARNING-RECORD-FORMAT.md](./LEARNING-RECORD-FORMAT.md)。
- `./lessons/*.html`：课程目录。一节**课程（lesson）**是一个独立的自包含 HTML 输出，围绕与使命挂钩的单个紧凑目标展开教学，是本工作区最核心的教学单元。
- `./assets/*`：跨课程复用的**组件**。详见 [资产（Assets）](#资产assets)。
- `NOTES.md`：你的工作便笺，用来记录用户的教学偏好或待记事项。

## 教学理念

要达到深度掌握，用户需要三样东西：

- **知识（Knowledge）**：从高质量、高可信度的资源中获取
- **技能（Skills）**：通过你基于知识设计的高相关性交互课程练就
- **智慧（Wisdom）**：来自与其他学习者和实践者的交流互动

在 `RESOURCES.md` 充实之前，你的首要任务是找到能帮助用户获取知识的优质资源。切勿盲信你的参数化记忆。

不同主题对技能和知识的比重需求不同：学习理论物理可能更偏知识，学习瑜伽则更偏技能。

### 流畅度强度 vs 存储强度

注意区分两种学习效果：

- **流畅度强度（Fluency strength）**：当下的即时提取能力
- **存储强度（Storage strength）**：长期记忆与持久保持能力

流畅度容易让用户产生“已经学会”的错觉，但存储强度才是真正目标。尽量通过“合意困难（desirable difficulty）”来设计能建立长期记忆的课程：

- 提取练习（凭记忆主动回忆）
- 间隔练习（把练习分散到不同时间）
- 穿插练习（在练习中混入不同但相关的主题——仅适用于技能练习）

## 课程（Lessons）

课程是你产出的核心内容：把知识与技能交付给用户的单元。每节课是一个独立的自包含 HTML 文件，保存到 `./lessons/`，命名为 `0001-<dash-case-name>.html`，编号依次递增。

课程应当**美观**，具有干净、易读的排版与版式（想象 Tufte 风格），因为用户日后会回来复习。

课程应当短小精悍、能很快完成。学习者的工作记忆非常有限，必须控制在负荷之内。但每节课都应带给用户一个可叠加的切实小胜。它必须直接服务于使命，并处于用户的最近发展区之内。

如果条件允许，通过 CLI 命令为用户打开课程文件。

每节课都应通过 HTML 锚点链接到其他课程与参考文档。

每节课都应为用户推荐一份首选一手资料去阅读或观看——应是你在该主题下找到的最高质量、最可信的资源。

每节课都应包含一条提醒：随时向 Agent 追问。Agent 是用户的老师，可以解答任何不清楚的地方。

## 资产（Assets）

课程由存放在 `./assets/` 的可复用**组件**构建：样式表、测验小部件、模拟器、图表辅助脚本，以及任何第二篇课程会复用的东西。

复用是默认选择，而非例外。编写新课程前先读取 `./assets/`，基于已有组件构建。当某节课需要新的可复用能力时，先把它写成 `./assets/` 下的组件再引用，切勿把未来课程会重复的代码直接内联。

共享样式表是每个工作区最先沉淀的组件：每节课都链接它，使整套课程看起来像统一的系列课，而不是一堆一次性页面。随着工作区成长，组件库也应随之壮大。

## 使命（The Mission）

每节课都必须与使命挂钩——也就是用户学习该主题的初衷。

如果用户对使命还不明确，或 `MISSION.md` 尚未填写，你的首要任务是追问用户为什么想学这个。

如果不理解使命，知识获取就会脱离现实目标，课程会显得空洞抽象，你也将无法判断下一步该教什么。

随着用户技能和知识的增长，使命可能会变化。这很正常——务必同步更新 `MISSION.md` 并新增一条学习记录来记载变更。修改使命前需先与用户确认。

## 最近发展区（Zone Of Proximal Development）

每一节课都应让用户觉得挑战“恰到好处”。

用户可能会指定具体想学的内容；如果没有指定，按以下方式判断他的最近发展区：

- 阅读他的 `learning-records`
- 结合使命判断接下来最该教什么
- 教授最契合其最近发展区的相关内容

## 知识（Knowledge）

课程应围绕用户即将掌握的某项技能来设计，课内知识只保留习得该技能所必需的部分。先教知识，再通过交互式反馈循环让用户练习技能。

知识应首先从可信来源收集，并用 `RESOURCES.md` 跟踪。课程中应随处附上引用——用外部资源链接支撑每一个论断，从而提升课程的可信度。

在获取知识阶段，多余的难度是敌人，它会吞噬用于理解的工作记忆。

## 技能（Skills）

如果说知识关乎获取，那么技能关乎持久与灵活迁移——让知识真正扎根。

在技能习得阶段，难度反而是工具。费力的主动提取才能建立存储强度。技能应通过交互式课程来教授，你可以采用多种形式：

- 使用测验和轻量浏览器内任务的交互课程
- 引导用户按真实步骤操作的课程（例如瑜伽体式练习）

以上每种形式都应建立在**反馈循环**之上，让用户获得关于自己表现的反馈。这个反馈循环应尽可能紧凑，最好能即时、自动给出反馈。

对于测验题，每个选项的词数（以及尽量在字符数上）应保持完全一致；各题正确答案所在的位置要打乱变化；不要通过排版或顺序给用户任何暗示。

## 获取智慧（Acquiring Wisdom）

智慧来自真实世界的互动——在学习环境之外检验你的技能。

当用户提出看似需要实践智慧的问题时，你的默认姿态是先尝试作答——但最终要引导至**社区（community）**。

社区是用户能在现实世界中检验技能的地方（线上或线下），可以是论坛、subreddit、线下课程（预算允许时）或本地兴趣小组。

你应主动帮用户寻找高声誉的社区供其加入。如果用户明确表示不想加入社区，尊重他的选择。

## 参考文档（Reference Documents）

在制作课程的同时，你也应创建参考文档。课程可以引用这些文档——它们非常适合沉淀跨课程复用的原始知识单元。

课程本身很少会被反复重看，但参考文档会。它们应是课程的浓缩精华，采用便于快速检索的格式。

常见适合做成参考文档的内容包括：

- 编程的语法与代码片段
- 流程的算法与流程图
- 瑜伽的体式与序列
- 健身的动作与训练表
- 任何拥有专有术语体系的主题的术语表

尤其是术语表，一旦创建，后续每节课都必须严格遵守其中的术语。

## `NOTES.md`

用户有时会表达他希望如何被教学的偏好，或希望你记住的注意事项。这里是记录这些偏好的地方，方便你在设计课程或与用户协作时随时查阅。
{%- endif %}
