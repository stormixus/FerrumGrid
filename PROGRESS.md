# FerrumGrid Plan v7 — Progress (2026-05-05)

## 오늘 완료 (12 commits, master branch)

### Plan v7 Phases
- Phase 1.95: grid.rs 9모듈 분리 (4957→155줄), StateOp dispatch 구현
- Phase 3: explicit tx tracking + dangling tx 30s/60s 정책
- Phase 4c: BI group-by 카드 + 바 차트
- DiagnosticsPanel: 5채널 wire + UI polish + channel filter + tracing

### UX 수정
- 자동완성: Tab/Enter/Up/Down 키보드 네비게이션, 팝업 dismiss, trailing space
- 탐색기: 선택된 스키마/테이블 하이라이트
- 오브젝트 뷰: sub_toolbar 레이아웃 버그 수정 (모든 뷰 컨텐트 안 보이던 원인)
- BI: Load Data 버튼 추가 (테이블 컨텍스트에서 자동 쿼리)
- 백업: 파일 브라우저 (디렉토리 스캔 + 테이블 표시 + Show/Delete 액션)
- 빌드: 0 warnings, 210 tests pass

## 다음 작업 (우선순위순)

### 1. 자동완성 커서 위치 확인 (높음)
- Tab 수락 후 커서가 완성된 단어 끝+공백으로 이동하는지 테스트
- TextEditState::load/store로 커서 세팅했지만 실제 동작 미확인
- 파일: src/ui/editor.rs (render_completion_popup, apply_completion 부근)

### 2. Model 탭 ER diagram UX (중)
- 스키마 선택 시 테이블/FK 로딩 중 스피너 표시
- 데이터 없을 때 안내 메시지 개선
- 파일: src/ui/er_diagram.rs (render_er_diagram, sync_schema_visualizer)

### 3. Automation 스케줄링 UI (중)
- 현재 3개 프리셋만 있음 (Vacuum, Reindex, Refresh Mat Views)
- 실제 작업 생성/예약/실행/취소 UI 필요
- 백엔드: src/automation/scheduler.rs (ScheduledTask, AutomationStore 이미 구현)
- UI: src/ui/objects/automation.rs (83줄 — 대폭 확장 필요)

### 4. 백업 자체 엔진 (중)
- pg_dump 의존 → SQL-only 자체 엔진 (information_schema + COPY TO)
- 또는 pg_dump 방식 유지하되 폴더 UI 개선

### 5. grid 모듈 line count 감축 (낮음)
- info_panel.rs: 1631 → ≤800 (JSON 에디터 분리)
- render.rs: 1558 → ≤800
- selection.rs: 1177 → ≤500

### 6. dispatch → grid 연결 (낮음)
- grid_dispatch.rs의 dispatch 함수가 구현됐지만 grid에서 미사용
- inline 입력 처리를 dispatch 시스템으로 전환 (리팩토링 리스크)

## 프로젝트 상태
- Branch: master
- 최신 커밋: 2f8343f (objects view layout fix)
- Build: cargo build → 0 errors, 0 warnings
- Tests: cargo test → 210 pass, 0 fail
