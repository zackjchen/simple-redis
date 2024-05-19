/*
我觉得encode的实现放在一起方便我读，我先不refactor成单独的文件
RESP data type	        Minimal protocol version	Category	      First byte
Simple strings	        RESP2	                    Simple	          +
Simple Errors	        RESP2	                    Simple	          -
Integers	            RESP2	                    Simple	          :
Bulk strings	        RESP2	                    Aggregate	      $
Arrays	                RESP2	                    Aggregate	      *
Nulls	                RESP3	                    Simple	          _
Booleans	            RESP3	                    Simple	          #
Doubles	                RESP3	                    Simple	          ,
Maps	                RESP3	                    Aggregate	      %
Sets	                RESP3	                    Aggregate	      ~


*/

// impl RespEncode for RespFrame; 由dispatch macro生成
