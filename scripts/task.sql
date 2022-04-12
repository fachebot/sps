-- ----------------------------
-- Table structure for task
-- ----------------------------
CREATE TABLE "task" (
  "id" int8 NOT NULL GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
  "message_id" int8 NOT NULL,
  "user_id" int8 NOT NULL,
  "transport" int8 NOT NULL,
  "state" varchar(16) NOT NULL,
  "retry_count" int4 NOT NULL,
  "reason" varchar(255),
  "creation_time" timestamptz(6) NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ----------------------------
-- Indexes structure for table task
-- ----------------------------
CREATE INDEX "idx_task_message_id" ON "task" USING btree ("message_id");
