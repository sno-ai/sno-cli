# Installer review closure

The original [source review](installer-code-review.md), [test review](installer-test-review.md), and [legacy assertion review](installer-legacy-test-review.md) are preserved. Each file received at most one review pass. Repairs were verified by targeted tests; no second clean independent verdict is claimed.

| Source finding | Disposition | Evidence |
|---|---|---|
| Missing utility must fail the entire install | **Possible FP; contract conflict.** PRD DEC-3 and DEC-4 explicitly require named dependent-skill skips for unavailable programs. Complete release acceptance still requires real runtime artifacts. The dispatcher confirmed that reading in its mailbox answer. No human-only ledger rejection was set and no fatal-error requirement was added. | [PRD decisions](../sno-assemble-PRD.md); original finding retained |
| Restrictive umask breaks ownership and rollback | Fixed by explicit temporary-file permissions before sync/rename. | restrictive_umask_preserves_modes_across_update_and_rollback |
| Crash after remove commit loses directory cleanup | Fixed by cleaning committed owned directories during recovery before journal deletion. | committed_remove_cleans_owned_directories_before_reassembly |
| Inline payload duplicates exceed journal capacity | Fixed with content-addressed durable bodies and metadata references; obsolete owned release paths are pruned. | large_program_update_and_recovery_keep_payload_out_of_journal_metadata |

All three admitted test-review findings were repaired. Their exact changes and regression coverage are recorded in [the test close](installer-test-close.md). The narrow legacy-test review found no issue. The original missing-utility finding remains visibly classified as Possible FP, not silently deleted or called an independently approved repair.
