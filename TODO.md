# Core Event Version Field - Implementation TODO

## Status: [ ] 0% Complete

### Breakdown of Approved Plan

1. **[x] Update src/lib.rs Constants**  
   - Defined `EVENT_SCHEMA_VERSION_V2 = 2u32`  
   - Renamed V1 events to V2 (`ofr_reg2`, `rv_init2`, etc.)  
   - Removed `EventVersioningEnabled` DataKey & `is_event_versioning_enabled()`  

2. **[x] Add Event Emission Helper in src/lib.rs**  
   - Added `emit_v2_event()` helper with IntoVal<Vec> support  
   - Used in register_offering + report_revenue v2 emissions  

3. **[ ] Update ALL Core Event Emissions in src/lib.rs**  
   - `register_offering`: `(EVENT_OFFER_REG_V2, issuer, ns) -> (2u32, token, bps, payout)`  
   - `report_revenue`: All variants (init/ovrd/rej) prefix version  
   - `deposit_revenue`: `(EVENT_REV_DEPOSIT_V2, ...) -> (2u32, payment_token, amount, period_id)`  
   - `claim`: `(EVENT_CLAIM_V2, ...) -> (2u32, holder, total_payout, periods_vec)`  
   - `set_holder_share`: Prefix version  
   - Multisig events, pause/freeze, blacklist/whitelist ALL get v2 variants  
   - Update EventIndexTopicV2.version to 2  

4. **[ ] Add NatSpec Comments in src/lib.rs**  
   ```rust
   /// Versioned event v2: [version:u32=2, token:Address, revenue_share_bps:u32, payout_asset:Address]
   env.events().publish(...);
   ```

5. **[ ] Update src/test.rs**  
   - Fix existing event assertions: check `data[0] == 2u32`  
   - **[NEW]** `test_version_field_deterministic()`: Always emits v2  
   - **[NEW]** `test_all_core_events_versioned()`: Smoke test every emission  
   - **[NEW]** `test_version_field_security()`: Malformed/replay rejection logic  
   - Fuzz version boundaries (0, u32::MAX)  

6. **[x] Create docs/core-event-version-field.md**  
   - ✅ Schema table for core v2 events  
   - ✅ Security assumptions + off-chain rejection logic  
   - ✅ Migration v1→v2 + backward compat  
   - ✅ Indexer best practices + verification steps  

7. **[ ] Validation**  
   - `cargo check`  
   - `cargo clippy`  
   - `cargo test`  
   - `cargo test --features testutils`  
   - Manual: Verify ALL events emit version=2  

8. **[ ] Git Workflow**  
   ```
   git checkout -b feature/contracts-043-core-event-version-field
   git add .
   git commit -m "feat: core event version field v2 deterministic emission"
   git push origin feature/contracts-043-core-event-version-field
   gh pr create --title "Core Event Version Field" --body "..." --base main
   ```

### Progress Tracking
- Complete step → Change `[ ]` to `[x]`  
- Current: Step 1

**Next Action**: Step 7 - Run `cargo check`, `cargo clippy`, `cargo test`, update test.rs with version assertions

