## Key

- :white_check_mark: - api provided
- :sparkle: - api internal used
- :arrow_down: - api is low priority; open an issue
- :x: - api is not supported
- :rocket: - experiment api


## API
| API | STATUS |
| - | - |
| RedisModule_Alloc | :sparkle: |
| RedisModule_Realloc | :arrow_down: |
| RedisModule_Free | :sparkle: |
| RedisModule_Strdup | :arrow_down: |
| RedisModule_GetApi | :arrow_down: |
| RedisModule_CreateCommand | :sparkle: |
| RedisModule_SetModuleAttribs | :arrow_down: |
| RedisModule_IsModuleNameBusy | :white_check_mark: |
| RedisModule_WrongArity | :sparkle: |
| RedisModule_ReplyWithLongLong | :sparkle: |
| RedisModule_GetSelectedDb | :white_check_mark: |
| RedisModule_SelectDb | :white_check_mark: |
| RedisModule_OpenKey | :white_check_mark: |
| RedisModule_CloseKey | :sparkle: |
| RedisModule_KeyType | :white_check_mark: |
| RedisModule_ValueLength | :sparkle: |
| RedisModule_ListPush | :white_check_mark: |
| RedisModule_ListPop | :white_check_mark: |
| RedisModule_Call | :white_check_mark: |
| RedisModule_CallReplyProto | :arrow_down: |
| RedisModule_FreeCallReply | :sparkle: |
| RedisModule_CallReplyType | :sparkle: |
| RedisModule_CallReplyInteger | :sparkle: |
| RedisModule_CallReplyLength | :sparkle: |
| RedisModule_CallReplyArrayElement | :sparkle: |
| RedisModule_CreateString | :sparkle: |
| RedisModule_CreateStringFromLongLong | :arrow_down: |
| RedisModule_CreateStringFromDouble | :arrow_down: |
| RedisModule_CreateStringFromLongDouble | :arrow_down: |
| RedisModule_CreateStringFromString | :arrow_down: |
| RedisModule_CreateStringPrintf | :arrow_down: |
| RedisModule_FreeString | :sparkle: |
| RedisModule_StringPtrLen | :white_check_mark: |
| RedisModule_ReplyWithError | :sparkle: |
| RedisModule_ReplyWithSimpleString | :sparkle: |
| RedisModule_ReplyWithArray | :sparkle: |
| RedisModule_ReplyWithNullArray | :sparkle: |
| RedisModule_ReplyWithEmptyArray | :sparkle: |
| RedisModule_ReplySetArrayLength | :sparkle: |
| RedisModule_ReplyWithStringBuffer | :sparkle: |
| RedisModule_ReplyWithCString | :arrow_down: |
| RedisModule_ReplyWithString | :sparkle: |
| RedisModule_ReplyWithEmptyString | :sparkle: |
| RedisModule_ReplyWithVerbatimString | :arrow_down: |
| RedisModule_ReplyWithNull | :sparkle: |
| RedisModule_ReplyWithDouble | :arrow_down: |
| RedisModule_ReplyWithLongDouble |  :sparkle: |
| RedisModule_ReplyWithCallReply | :arrow_down: |
| RedisModule_StringToDouble | :arrow_down: |
| RedisModule_StringToLongDouble | :sparkle: |
| RedisModule_AutoMemory | :arrow_down: |
| RedisModule_Replicate | :white_check_mark: |
| RedisModule_ReplicateVerbatim | :white_check_mark: |
| RedisModule_CallReplyStringPtr | :sparkle: |
| RedisModule_CreateStringFromCallReply |:arrow_down: |
| RedisModule_DeleteKey  | :white_check_mark: |
| RedisModule_UnlinkKey | :white_check_mark: |
| RedisModule_StringSet | :white_check_mark: |
| RedisModule_StringDMA | :sparkle: |
| RedisModule_StringTruncate | :arrow_down: |
| RedisModule_GetExpire | :white_check_mark: |
| RedisModule_SetExpire | :white_check_mark: |
| RedisModule_ResetDataset | :white_check_mark: |
| RedisModule_DbSize | :white_check_mark: |
| RedisModule_RandomKey | :arrow_down: |
| RedisModule_ZsetAdd | :white_check_mark: |
| RedisModule_ZsetIncrby | :white_check_mark: |
| RedisModule_ZsetScore | :white_check_mark: |
| RedisModule_ZsetRem | :white_check_mark: |
| RedisModule_ZsetRangeStop | :sparkle: |
| RedisModule_ZsetFirstInScoreRange | :white_check_mark: |
| RedisModule_ZsetLastInScoreRange | :white_check_mark: |
| RedisModule_ZsetFirstInLexRange | :white_check_mark: |
| RedisModule_ZsetLastInLexRange | :white_check_mark: |
| RedisModule_ZsetRangeCurrentElement | :sparkle: |
| RedisModule_ZsetRangeNext | :sparkle: |
| RedisModule_ZsetRangePrev | :sparkle: |
| RedisModule_ZsetRangeEndReached | :sparkle: |
| RedisModule_HashSet | :white_check_mark: |
| RedisModule_HashGet | :white_check_mark: |
| RedisModule_IsKeysPositionRequest | :white_check_mark: |
| RedisModule_KeyAtPos | :white_check_mark: |
| RedisModule_GetClientId | :white_check_mark: |
| RedisModule_GetClientInfoById | :white_check_mark: |
| RedisModule_PublishMessage | :white_check_mark: |
| RedisModule_GetContextFlags | :white_check_mark: |
| RedisModule_AvoidReplicaTraffic | :white_check_mark: |
| RedisModule_PoolAlloc | :arrow_down: |
| RedisModule_CreateDataType | :white_check_mark: |
| RedisModule_ModuleTypeSetValue | :white_check_mark: |
| RedisModule_ModuleTypeReplaceValue | :white_check_mark: |
| RedisModule_ModuleTypeGetType | :white_check_mark: |
| RedisModule_ModuleTypeGetValue | :white_check_mark: |
| RedisModule_IsIOError | :white_check_mark: |
| RedisModule_SetModuleOptions | :white_check_mark: |
| RedisModule_SignalModifiedKey | :white_check_mark: |
| RedisModule_SaveUnsigned | :white_check_mark: |
| RedisModule_LoadUnsigned | :white_check_mark: |
| RedisModule_SaveSigned | :white_check_mark: |
| RedisModule_LoadSigned | :white_check_mark: |
| RedisModule_EmitAOF | :white_check_mark: |
| RedisModule_SaveString | :white_check_mark: |
| RedisModule_SaveStringBuffer | :white_check_mark: |
| RedisModule_LoadString | :white_check_mark: |
| RedisModule_LoadStringBuffer | :white_check_mark: |
| RedisModule_SaveDouble | :white_check_mark: |
| RedisModule_LoadDouble | :white_check_mark: |
| RedisModule_SaveFloat | :white_check_mark: |
| RedisModule_LoadFloat | :white_check_mark: |
| RedisModule_SaveLongDouble | :white_check_mark: |
| RedisModule_LoadLongDouble | :white_check_mark: |
| RedisModule_LoadDataTypeFromString | :white_check_mark: |
| RedisModule_SaveDataTypeToString | :white_check_mark: |
| RedisModule_Log | :white_check_mark: |
| RedisModule_LogIOError | :white_check_mark: |
| RedisModule__Assert | :arrow_down: |
| RedisModule_LatencyAddSample | :white_check_mark: |
| RedisModule_StringAppendBuffer | :white_check_mark: |
| RedisModule_RetainString | :arrow_down: |
| RedisModule_StringCompare | :arrow_down: |
| RedisModule_GetContextFromIO | :white_check_mark: |
| RedisModule_GetKeyNameFromIO | :white_check_mark: |
| RedisModule_GetKeyNameFromModuleKey | :white_check_mark: |
| RedisModule_Milliseconds | :white_check_mark: |
| RedisModule_DigestAddStringBuffer | :white_check_mark:  |
| RedisModule_DigestAddLongLong | :white_check_mark: |
| RedisModule_DigestEndSequence | :white_check_mark: |
| RedisModule_CreateDict | :white_check_mark: |
| RedisModule_FreeDict | :x: |
| RedisModule_DictSize | :x: |
| RedisModule_DictSetC | :x: |
| RedisModule_DictReplaceC | :x: |
| RedisModule_DictSet | :x: |
| RedisModule_DictReplace | :x: |
| RedisModule_DictGetC | :x: |
| RedisModule_DictGet | :x: |
| RedisModule_DictDelC | :x: |
| RedisModule_DictDel | :x: |
| RedisModule_DictIteratorStartC | :x: |
| RedisModule_DictIteratorStart | :x: |
| RedisModule_DictIteratorStop | :x: |
| RedisModule_DictIteratorReseekC | :x: |
| RedisModule_DictIteratorReseek | :x: |
| RedisModule_DictNextC | :x: |
| RedisModule_DictPrevC | :x: |
| RedisModule_DictNext | :x: |
| RedisModule_DictPrev | :x: |
| RedisModule_DictCompareC | :x: |
| RedisModule_DictCompare | :x: |
| RedisModule_RegisterInfoFunc | :x: |
| RedisModule_InfoAddSection | :x: |
| RedisModule_InfoBeginDictField | :x: |
| RedisModule_InfoEndDictField | :x: |
| RedisModule_InfoAddFieldString | :x: |
| RedisModule_InfoAddFieldCString | :x: |
| RedisModule_InfoAddFieldDouble | :x: |
| RedisModule_InfoAddFieldLongLong | :x: |
| RedisModule_InfoAddFieldULongLong | :x: |
| RedisModule_GetServerInfo | :x: |
| RedisModule_FreeServerInfo | :x: |
| RedisModule_ServerInfoGetField | :x: |
| RedisModule_ServerInfoGetFieldC | :x: |
| RedisModule_ServerInfoGetFieldSigned | :x: |
| RedisModule_ServerInfoGetFieldUnsigned | :x: |
| RedisModule_ServerInfoGetFieldDouble | :x: |
| RedisModule_SubscribeToServerEvent | :white_check_mark: |
| RedisModule_SetLRU | :white_check_mark: |
| RedisModule_GetLRU | :white_check_mark: |
| RedisModule_SetLFU | :white_check_mark: |
| RedisModule_SetLFU | :white_check_mark: |
| RedisModule_BlockClientOnKeys | :white_check_mark: |
| RedisModule_SignalKeyAsReady | :white_check_mark: |
| RedisModule_GetBlockedClientReadyKey | :white_check_mark: |
| RedisModule_ScanCursorCreate | :white_check_mark: |
| RedisModule_ScanCursorRestart | :white_check_mark: |
| RedisModule_ScanCursorDestroy | :sparkle: |
| RedisModule_Scan | :white_check_mark: |
| RedisModule_ScanKey | :white_check_mark: |
| RedisModule_BlockClient | :white_check_mark:  :rocket: |
| RedisModule_UnblockClient | :white_check_mark:  :rocket: |
| RedisModule_IsBlockedReplyRequest | :white_check_mark:  :rocket: |
| RedisModule_IsBlockedTimeoutRequest | :white_check_mark:  :rocket: |
| RedisModule_GetBlockedClientPrivateData | :white_check_mark:  :rocket: |
| RedisModule_GetBlockedClientHandle | :sparkle:  :rocket: |
| RedisModule_AbortBlock | :white_check_mark:  :rocket: |
| RedisModule_GetThreadSafeContext | :white_check_mark:  :rocket: |
| RedisModule_FreeThreadSafeContext | :white_check_mark:  :rocket: |
| RedisModule_ThreadSafeContextLock | :white_check_mark:  :rocket: |
| RedisModule_ThreadSafeContextUnlock | :white_check_mark:  :rocket: |
| RedisModule_SubscribeToKeyspaceEvents | :white_check_mark:  :rocket: |
| RedisModule_NotifyKeyspaceEvent | :white_check_mark:  :rocket: |
| RedisModule_GetNotifyKeyspaceEvents | :white_check_mark:  :rocket: |
| RedisModule_RegisterClusterMessageReceiver | :white_check_mark:  :rocket: |
| RedisModule_SendClusterMessage | :white_check_mark:  :rocket: |
| RedisModule_GetClusterNodeInfo | :white_check_mark:  :rocket: |
| RedisModule_GetClusterNodesList | :white_check_mark:  :rocket: |
| RedisModule_FreeClusterNodesList | :sparkle:  :rocket: |
| RedisModule_CreateTimer | :white_check_mark:  :rocket: |
| RedisModule_StopTimer | :white_check_mark:  :rocket: |
| RedisModule_GetTimerInfo | :white_check_mark:  :rocket: |
| RedisModule_GetMyClusterID | :white_check_mark:  :rocket: |
| RedisModule_GetClusterSize | :white_check_mark:  :rocket: |
| RedisModule_GetRandomBytes | :arrow_down:  :rocket: |
| RedisModule_GetRandomHexChars | :arrow_down:  :rocket: |
| RedisModule_SetDisconnectCallback | :white_check_mark:  :rocket: |
| RedisModule_SetClusterFlags | :white_check_mark:  :rocket: |
| RedisModule_ExportSharedAPI | :white_check_mark:  :rocket: |
| RedisModule_GetSharedAPI | :white_check_mark:  :rocket: |
| RedisModule_RegisterCommandFilter | :x:  :rocket: |
| RedisModule_UnregisterCommandFilter | :x:  :rocket: |
| RedisModule_CommandFilterArgsCount | :x:  :rocket: |
| RedisModule_CommandFilterArgGet | :x:  :rocket: |
| RedisModule_CommandFilterArgInsert | :x:  :rocket: |
| RedisModule_CommandFilterArgReplace | :x:  :rocket: |
| RedisModule_CommandFilterArgDelete | :x:  :rocket: |
| RedisModule_Fork | :x:  :rocket: |
| RedisModule_ExitFromChild | :x:  :rocket: |
| RedisModule_KillForkChild | :x:  :rocket: |
| RedisModule_GetUsedMemoryRatio | :white_check_mark:  :rocket: |
| RedisModule_MallocSize | :arrow_down:  :rocket: |
| RedisModule_CreateModuleUser | :white_check_mark:  :rocket: |
| RedisModule_FreeModuleUser | :white_check_mark:  :rocket: |
| RedisModule_SetModuleUserACL | Provide  :rocket: |
| RedisModule_AuthenticateClientWithACLUser | :white_check_mark:  :rocket: |
| RedisModule_AuthenticateClientWithUser | :white_check_mark:  :rocket: |
| RedisModule_DeauthenticateAndCloseClient | :white_check_mark:  :rocket: |