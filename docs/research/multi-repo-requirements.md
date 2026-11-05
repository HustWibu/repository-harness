# Yêu cầu: repository-harness cho hệ microservices nhiều repo

Status: research, not a release gate.

Phạm vi: mở rộng `repository-harness` để agent làm việc đúng trong một hệ gồm nhiều
repo service độc lập, nơi contract, config và tiến độ nằm **giữa** các repo. Mọi
yêu cầu giữ nguyên triết lý hiện tại: harness cung cấp giao thức, template và skill
gọi tường minh, còn sự thật (lệnh, config, contract, trạng thái) thuộc về repo dùng
harness. Mỗi yêu cầu có cột **Kiểm** là cách chứng minh đã đạt. Câu chữ nằm trong
payload viết bằng tiếng Anh; phần giải thích viết bằng tiếng Việt.

Ràng buộc chung `CON-1` đến `CON-10` và điều kiện hoàn thành `DONE-1` đến `DONE-6`
trong [`harness-requirements-ideas.md`](harness-requirements-ideas.md) áp nguyên cho
tài liệu này; bỏ phần nói về Windows vì decision 0032 đã giới hạn ở Linux và macOS.

## Vấn đề

Harness hiện giả định **ranh giới hệ thống trùng với git root**. Trong hệ
microservices giả định đó sai ở năm chỗ:

1. Hiểu một service cần biết ai gửi cho nó và nó gửi cho ai, mà câu trả lời nằm ở
   repo khác.
2. Contract (topic + schema message, `.proto`, endpoint HTTP) nằm giữa các repo, có
   khi bị chép tay qua nhiều repo.
3. Config chạy thật nằm ở repo config riêng, tách theo môi trường; config trong repo
   service thường là của môi trường dev.
4. Một việc chạm nhiều repo, nên plan và handoff ghi theo một repo bị chia nhỏ.
5. Bằng chứng trong một repo (unit test xanh) không chứng minh được hành vi ở service
   bên cạnh.

Ví dụ tham chiếu: một hệ khoảng 45 repo service, giao tiếp chủ yếu qua Kafka, config
theo môi trường nằm ở hai repo config riêng, toolchain trộn lẫn (JDK 8 và 21), test
gần như bằng 0.

## Hiện trạng đã đối chiếu với repo (commit `cdb0d23`, 2026-11-03)

| Điểm | Hiện trạng | Ảnh hưởng tới yêu cầu |
| --- | --- | --- |
| Khái niệm nhiều repo | `grep` các từ multi-repo, polyrepo, microservice, cross-repo, sibling trong `docs/`, `crates/`, `.claude/` không có kết quả liên quan. Gần nhất là một câu trong `docs/WORKFLOW.md`: báo capability thiếu kèm "owning repository or external dependency" | Có điểm tựa sẵn để mở rộng, không phải dựng khái niệm mới |
| Ngân sách lời vào cửa | `scripts/claude-harness-block.md` + `docs/WORKFLOW.md` = 533 từ, trần 1000 (`scripts/context-budget.tsv`) | Phần thêm vào `WORKFLOW.md` phải ngắn. Xem MR-0 |
| `onboard-repository/SKILL.md` | 27.423 byte, trần grandfathered 27.500, chỉ được giảm | Mọi bổ sung cho skill này phải nằm trong `references/`. Xem MR-ONB-2 |
| `encode-invariant/SKILL.md` | 3.291 byte, trần mặc định 6.000 | Còn chỗ, nhưng ví dụ dài vẫn nên nằm ở pattern doc |
| Template plan và handoff | `exec-plan.md` 899 byte, `session-handoff.md` 964 byte; không có chỗ ghi nhiều repo | Thêm bảng repo chạm tới. Xem MR-STATE |
| Chế độ cài | Flag installer: `--merge`, `--override`, `--refresh-claude-shim`, `--dry-run`, `--with-engineering-wisdom`… Không có flag nào giữ file đã cài ra ngoài `git status` | Xem MR-LOCAL |
| Decision mới nhất | `0032-linux-and-macos-only.md` | Decision mới bắt đầu từ 0033 |

## Những gì cố ý không làm

Ba nhu cầu thường gặp ở hệ microservices **không** đưa vào harness, vì trái với quyết
định đã có:

| Nhu cầu | Lý do không làm | Thay bằng |
| --- | --- | --- |
| `init.sh` tự chọn toolchain, script verify nhiều mức | Gap analysis C-1, decision 0029: harness không ship lệnh, consumer sở hữu lệnh verify | MR-DONE: nguyên tắc ghi phạm vi bằng chứng; consumer tự mã hoá cổng qua `/encode-invariant` |
| Theo dõi tiến độ tính năng xuyên repo | Gap analysis C-3, decision 0027: không làm lại task database | MR-STATE: bảng repo chạm tới trong plan và handoff, plan nằm ở một repo chủ |
| Đọc log, metric của môi trường triển khai | Gap analysis C-5: runtime thuộc consumer | Mục Runtime Evidence trong `application-runbook.md` ghi rõ nguồn log/metric của consumer |

## MR-0. Ràng buộc riêng của tài liệu này

| ID | Ràng buộc | Kiểm |
| --- | --- | --- |
| MR-0.1 | Hệ chỉ có một repo không bị ảnh hưởng: mọi mục mới là tuỳ chọn, để trống thì hành vi như cũ. | Cài mới vào repo đơn lẻ, không có câu hỏi hay file bắt buộc nào thêm; test cài đặt hiện có pass |
| MR-0.2 | Phần thêm vào `docs/WORKFLOW.md` cho cả tài liệu này không vượt 150 từ, để tổng lời vào cửa vẫn dưới trần 1000. | `scripts/check-context-budget.sh` pass; đo lại số từ trước và sau |
| MR-0.3 | Harness không đọc, ghi hay gọi repo khác khi cài, cập nhật hay chạy lệnh. Chỉ skill do người dùng gọi mới được đọc repo lân cận, và chỉ đọc. | Đọc diff của `crates/`, không có truy cập ngoài thư mục đích; skill ghi rõ quyền chỉ đọc |
| MR-0.4 | Không giả định công nghệ giao tiếp. Kafka, gRPC, HTTP chỉ là ví dụ trong template; không có trường bắt buộc gắn với một công nghệ. | Đọc template, mọi trường đều trung lập công nghệ |

## MR-MAP. Repo liên quan và nguồn thẩm quyền bên ngoài

Vấn đề: agent không biết sự thật nào của repo này nằm ở repo khác, nên hoặc đoán, hoặc
coi config trong repo là sự thật.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-MAP-1 | `docs/README.md` có mục tuỳ chọn *Related repositories*: mỗi dòng gồm tên repo, vai trò đối với repo này (contract, config theo môi trường, spec, thư viện dùng chung), và vị trí (URL hoặc đường dẫn tương đối nếu cùng workspace). | Đọc file payload, có mục kèm hướng dẫn điền; để trống không phát sinh lỗi |
| MR-MAP-2 | Mục đó có bảng *Authority by truth type*: loại sự thật (contract, config theo môi trường, yêu cầu, tiến độ) → repo và đường dẫn là nguồn gốc. Mỗi loại chỉ có một nguồn gốc; nơi khác chỉ trỏ tới. | Đọc template; hướng dẫn nói rõ quy tắc một nguồn gốc |
| MR-MAP-3 | `docs/WORKFLOW.md` §Missing Capabilities thêm một câu: khi câu trả lời thuộc repo khác, ghi repo sở hữu theo mục Related repositories thay vì để Unknown không rõ chủ. | Đọc file; MR-0.2 pass |
| MR-MAP-4 | Đường dẫn trong mục này không được là đường dẫn tuyệt đối riêng của máy. | Test kiểm docs từ chối đường dẫn bắt đầu bằng thư mục home trong file payload mẫu |

## MR-CFG. Config nằm ngoài repo

Vấn đề: `WORKFLOW.md` đã nói "configurable defaults are not authority", nhưng chưa nói
config trong repo có thể hoàn toàn không phải config chạy thật.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-CFG-1 | `docs/WORKFLOW.md` §Authority thêm nguyên tắc: config trong repo có thể chỉ là của môi trường phát triển; trước khi kết luận về hành vi của một môi trường, xác định nguồn config của môi trường đó qua mục Authority by truth type. | Đọc file; MR-0.2 pass |
| MR-CFG-2 | `application-runbook.md` có bảng tuỳ chọn *Environments*: môi trường → nguồn config → cách triển khai → nguồn log/metric. Chỉ là cấu trúc, không có giá trị mẫu của một hệ cụ thể. | Đọc template; CON-5 pass |
| MR-CFG-3 | Nguyên tắc verify: thử với config của môi trường đích, chỉ thay host và secret; không tự thêm key thiếu để cho chạy được. Key thiếu là một phát hiện phải báo. | Đọc `WORKFLOW.md` hoặc `application-runbook.md`, có câu này |

## MR-CON. Contract giao tiếp giữa các repo

Vấn đề: thay đổi contract ở một repo làm hỏng repo khác mà không test nào trong repo
phát hiện.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-CON-1 | Có template tuỳ chọn `docs/templates/interface-contract.md`: tên contract, hướng (produce hoặc consume), nơi định nghĩa (đường dẫn file hoặc key config), repo sở hữu, các bên tiêu thụ đã biết, cách kiểm tương thích, mức độ đổi được phép. | File có trong `scripts/harness-install-files.txt` và trong binary; CON-4, CON-8 pass |
| MR-CON-2 | Template ghi rõ: danh sách producer/consumer nên được **sinh** từ nguồn thật (config, code) bằng công cụ của consumer, không viết tay; nếu viết tay thì ghi ngày đối chiếu. | Đọc template |
| MR-CON-3 | `docs/README.md` trỏ tới thư mục contract nếu consumer dùng template này. | Đọc file payload |

## MR-INV. Invariant xuyên repo

Vấn đề: `/encode-invariant` hiện chỉ nghĩ tới luật kiểm được trong một repo.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-INV-1 | `docs/patterns/encoding-invariants.md` có mục *Cross-repository invariants* với ít nhất hai mẫu: các bản chép của một contract phải giống nhau; tên kênh giao tiếp mà code dùng phải tồn tại trong config của từng môi trường. | Đọc pattern doc |
| MR-INV-2 | Mục đó nêu nơi đặt cổng: ở repo sở hữu contract hoặc repo spec dùng chung, chạy trên checkout của các repo liên quan; không đặt trong harness. | Đọc pattern doc; CON-5 pass |
| MR-INV-3 | `/encode-invariant` chấp nhận nguồn thẩm quyền nằm ở repo khác nếu nguồn đó được khai trong mục Authority by truth type, và ghi rõ repo + đường dẫn của nguồn. | Đọc `SKILL.md`; kích thước vẫn trong trần mặc định |
| MR-INV-4 | Mỗi mẫu có chứng minh hai chiều trong tài liệu: ca khớp thì pass, ca lệch cố ý thì fail và chỉ ra file lệch. | Đọc pattern doc, mỗi mẫu có cả hai ca |

## MR-DONE. Phạm vi của bằng chứng

Vấn đề: Definition of Done hiện có ba danh sách (verified, not verified, known broken)
nhưng không nói bằng chứng phủ tới đâu. "Test xanh" trong một repo dễ bị đọc thành
"hệ chạy đúng".

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-DONE-1 | `docs/WORKFLOW.md` §Definition Of Done: mỗi mục *verified* ghi phạm vi bằng chứng — trong repo, qua contract, hoặc trên môi trường triển khai. | Đọc file; MR-0.2 pass |
| MR-DONE-2 | Thay đổi chạm contract mà chỉ có bằng chứng trong repo thì phía tiêu thụ được liệt kê ở *not verified*, kèm lý do. | Đọc file, có câu này |
| MR-DONE-3 | `docs/templates/evaluator-rubric.md` có một tiêu chí chấm việc báo cáo có ghi phạm vi bằng chứng hay không. | Đọc template |
| MR-DONE-4 | Không có mức bắt buộc nào do harness áp đặt; mức tối thiểu theo loại thay đổi do consumer quyết định và ghi trong repo của họ. | Đọc file; không có câu bắt buộc mức cụ thể |

## MR-STATE. Plan và handoff nhiều repo

Vấn đề: một việc chạm nhiều repo; plan và handoff ghi trong một repo không cho session
sau biết các nhánh, PR đang mở ở repo khác.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-STATE-1 | `docs/templates/exec-plan.md` có bảng tuỳ chọn *Repositories touched*: repo, nhánh, PR hoặc thay đổi, trạng thái, bằng chứng đã có. | Đọc template |
| MR-STATE-2 | Template nêu quy tắc: plan nằm ở **một** repo chủ (repo spec dùng chung nếu có, không thì repo chạm nhiều nhất); repo khác chỉ trỏ tới, không chép plan. | Đọc template |
| MR-STATE-3 | `docs/templates/session-handoff.md` có cùng bảng, cộng mục việc còn dở theo từng repo. | Đọc template |
| MR-STATE-4 | Không thêm trạng thái máy đọc hay lệnh cập nhật trạng thái (giữ C-3). | Diff không có file trạng thái mới hay lệnh mới |

## MR-ONB. Onboarding đi qua ranh giới repo

Vấn đề: `/onboard-repository` lần theo một luồng vận hành; khi luồng rời repo qua message
hay gọi mạng, skill chỉ có thể ghi Unknown.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-ONB-1 | Khi luồng được lần theo rời repo, skill ghi một *boundary edge*: kênh (tên topic, endpoint, RPC), hướng, nơi khai báo trong repo này, repo phía bên kia nếu xác định được, và mức chắc chắn. | Chạy skill trên một repo service có gửi message sang repo khác; báo cáo có boundary edge với đủ trường |
| MR-ONB-2 | Hướng dẫn cho boundary edge nằm trong một file `references/` mới, `SKILL.md` chỉ thêm một dòng trỏ tới kèm điều kiện mở. | `SKILL.md` của skill vẫn ≤ 27.500 byte và không tăng quá số byte của dòng trỏ; `scripts/check-context-budget.sh` pass |
| MR-ONB-3 | Nếu repo phía bên kia có trong cùng workspace và người dùng cho phép, skill được đọc nó ở chế độ chỉ đọc để xác nhận cạnh; không thì để cạnh ở trạng thái chưa xác nhận. | Đọc reference; chạy skill không có quyền đọc repo bên kia, cạnh ở trạng thái chưa xác nhận |
| MR-ONB-4 | Đề xuất cải thiện tri thức mà skill đưa ra có thể nhắm tới mục Related repositories và Authority by truth type. | Đọc reference; chạy thử có đề xuất điền mục đó khi phát hiện repo liên quan |

## MR-LOCAL. Cài chỉ ở máy

Vấn đề: người dùng muốn dùng harness trên repo của team mà không commit file harness
vào repo chung.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-LOCAL-1 | Installer có flag `--local-only`: cài như bình thường, rồi ghi mọi đường dẫn đã cài vào `.git/info/exclude` của repo đích. | Test: cài vào repo git sạch với flag, `git status --porcelain` rỗng |
| MR-LOCAL-2 | Với file đã được git track (vd `CLAUDE.md` đã commit), flag không ghi đè mà báo và bỏ qua, trừ khi người dùng thêm `--refresh-claude-shim`; khi đó thay đổi hiện trong `git status` và báo rõ. | Test với `CLAUDE.md` đã track: không có thay đổi; thêm flag thì có đúng một file đổi và có cảnh báo |
| MR-LOCAL-3 | `harness update` và `harness doctor` nhận biết bản cài local-only và giữ các dòng exclude. | Test: update bản cài local-only, `git status` vẫn sạch |
| MR-LOCAL-4 | Gỡ chế độ local-only chỉ xoá đúng các dòng harness đã thêm vào exclude. | Test: exclude có dòng của người dùng, sau khi gỡ dòng đó còn nguyên |
| MR-LOCAL-5 | Thư mục đích không phải repo git thì flag báo lỗi rõ ràng, không cài. | Test với thư mục thường |

## MR-WS. Workspace chứa nhiều repo

Vấn đề: hệ nhiều repo thường được clone chung vào một thư mục cha không phải repo git.
Claude Code tự nạp `CLAUDE.md` của thư mục cha, nên đó là chỗ tự nhiên cho bản đồ cấp
hệ, nhưng updater hiện giả định một git root.

Nhóm này đổi mô hình sản phẩm nên **chỉ viết decision trước, chưa hiện thực**.

| ID | Yêu cầu | Kiểm |
| --- | --- | --- |
| MR-WS-1 | Có decision 0033 (Proposed) cân nhắc profile workspace: bản đồ cấp hệ ở thư mục cha, mỗi repo một shim ngắn trỏ lên; nêu phương án bị bác và lý do. | Có file decision dùng frontmatter của AUTH |
| MR-WS-2 | Decision trả lời: updater lưu `.harness-core/` ở đâu khi thư mục cha không phải git; xử lý conflict và recovery thế nào khi không có lịch sử git. | Đọc decision |
| MR-WS-3 | Decision so sánh với phương án không làm gì: người dùng tự viết bản đồ cấp hệ, harness chỉ cài vào từng repo kèm MR-MAP. | Đọc decision |

## Thứ tự đề xuất

1. **MR-MAP, MR-CFG, MR-DONE**: chỉ sửa `docs/README.md`, `docs/WORKFLOW.md`, hai
   template. Rẻ nhất, dùng chung ngân sách MR-0.2.
2. **MR-STATE, MR-CON**: template, không đụng code.
3. **MR-INV, MR-ONB**: pattern doc và skill; cần chạy thử trên một repo service thật.
4. **MR-LOCAL**: code Rust trong installer và updater, cần decision riêng theo CON-9.
5. **MR-WS**: chỉ decision.

Mỗi nhóm một PR, theo DONE-3.

## Backlog, chưa cần làm

| Ý | Điều kiện để đưa vào làm |
| --- | --- |
| Lệnh `harness doctor` kiểm các repo trong Related repositories có tồn tại và truy cập được | Khi MR-MAP được dùng thật và có lỗi do khai sai |
| Sinh mục Related repositories từ dấu vết trong code và config | Khi có ít nhất hai hệ dùng MR-MAP và cách khai đã ổn định |
| Mẫu boundary edge cho từng công nghệ giao tiếp | Khi MR-ONB chạy trên vài hệ và thấy trường chung không đủ |

## Câu hỏi cần làm rõ

| # | Câu hỏi | Ảnh hưởng tới |
| --- | --- | --- |
| Q1 | Harness có nên hỗ trợ hệ nhiều repo không, hay giữ phạm vi một repo và để người dùng tự xử lý phần giữa các repo? | Toàn bộ tài liệu |
| Q2 | Mục Related repositories đặt trong `docs/README.md` hay thành file riêng? File riêng dễ tìm hơn nhưng thêm một file vào payload. | MR-MAP |
| Q3 | Trần 150 từ thêm vào `WORKFLOW.md` có hợp lý không, hay nên đưa phần nhiều repo ra một file chỉ đọc khi cần? | MR-0.2, MR-CFG, MR-DONE |
| Q4 | Plan nhiều repo đặt ở repo chủ nào khi hệ không có repo spec dùng chung? | MR-STATE-2 |
| Q5 | `--local-only` có cần decision riêng không, hay coi là một chế độ cài thuộc decision 0020? | MR-LOCAL |
| Q6 | Profile workspace (MR-WS) có đáng làm không, khi Claude Code đã tự nạp `CLAUDE.md` của thư mục cha? | MR-WS |
| Q7 | Skill được đọc repo lân cận (MR-ONB-3) có trái với nguyên tắc chỉ thao tác trong repo đích không? | MR-ONB, MR-0.3 |

## Ngoài phạm vi

Script verify hay khởi tạo toolchain, theo dõi tiến độ dạng máy đọc, đọc log hoặc
metric của môi trường triển khai, điều phối deploy nhiều service, dịch vụ đăng ký
contract chạy nền. Các thứ này trái với C-1, C-3, C-5 và CON-3.
