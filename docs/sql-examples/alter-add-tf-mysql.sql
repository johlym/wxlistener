-- Add multi-channel temp-probe (WN34/WN34S ITEM_TF_USR) columns to an existing MySQL table.
-- Usage: mysql -u root -p weather < docs/sql-examples/alter-add-tf-mysql.sql
-- Note: MySQL does not support IF NOT EXISTS for ADD COLUMN on older versions;
--       re-running may error if columns already exist.

ALTER TABLE wx_records
  ADD COLUMN tf_temp_ch1 DOUBLE NULL,
  ADD COLUMN tf_temp_ch2 DOUBLE NULL,
  ADD COLUMN tf_temp_ch3 DOUBLE NULL,
  ADD COLUMN tf_temp_ch4 DOUBLE NULL,
  ADD COLUMN tf_temp_ch5 DOUBLE NULL,
  ADD COLUMN tf_temp_ch6 DOUBLE NULL,
  ADD COLUMN tf_temp_ch7 DOUBLE NULL,
  ADD COLUMN tf_temp_ch8 DOUBLE NULL,
  ADD COLUMN tf_battery_ch1 DOUBLE NULL,
  ADD COLUMN tf_battery_ch2 DOUBLE NULL,
  ADD COLUMN tf_battery_ch3 DOUBLE NULL,
  ADD COLUMN tf_battery_ch4 DOUBLE NULL,
  ADD COLUMN tf_battery_ch5 DOUBLE NULL,
  ADD COLUMN tf_battery_ch6 DOUBLE NULL,
  ADD COLUMN tf_battery_ch7 DOUBLE NULL,
  ADD COLUMN tf_battery_ch8 DOUBLE NULL;
