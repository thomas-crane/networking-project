p_lrdp = Proto("lrdp", "Lightweight Reliable Datagram Protocol")
local f_data_flag = ProtoField.new("Data flag", "lrdp.data_flag", ftypes.BOOLEAN, nil, base.DEC, 128)
local f_ack_flag = ProtoField.new("Ack flag", "lrdp.ack_flag", ftypes.BOOLEAN, nil, base.DEC, 64)
local f_seq_num = ProtoField.new("Sequence number", "lrdp.seq_num", ftypes.UINT8, nil, base.DEC, 56)
local f_ack_num = ProtoField.new("Acknowledgement number", "lrdp.ack_num", ftypes.UINT8, nil, base.DEC, 7)
local f_data = ProtoField.new("Data", "lrdp.data", ftypes.STRING)

p_lrdp.fields = {
  f_data_flag,
  f_ack_flag,
  f_seq_num,
  f_ack_num,
  f_data
}

function p_lrdp.dissector (buf, pkt, root)
  if buf:len() == 0 then return end

  pkt.cols.protocol = p_lrdp.name

  -- create a subtree for the protocol
  subtree = root:add(p_lrdp, buf(0))

  -- add flags.
  subtree:add(f_data_flag, buf(0, 1))
  subtree:add(f_ack_flag, buf(0, 1))

  -- seq num.
  subtree:add(f_seq_num, buf(0, 1))

  -- ack num.
  subtree:add(f_ack_num, buf(0, 1))

  -- data.
  subtree:add(f_data, buf(1, -1))

end

function p_lrdp.init()
end

-- register the dissector for UDP port 6860.
local udp_dissector_table = DissectorTable.get("udp.port")
dissector = udp_dissector_table:get_dissector(6860)
udp_dissector_table:add(6860, p_lrdp)
