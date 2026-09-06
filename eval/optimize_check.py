import sys,json,time
sys.path.insert(0,'eval')
import bench
rows=[]
port=bench.free_port()
p,_=bench.start_server(bench.FILES['large-v3-q5_0'],True,port)
try:
    items=[x for x in bench.load_items() if x['language']=='el']
    for lang,beam in [('auto',5),('el',5),('el',1)]:
        for it in items:
            text,ms,_=bench.infer(port,it['path'],lang,None,beam)
            rows.append(dict(id=it['id'],language=lang,beam=beam,text=text,ms=ms,wer=bench.jiwer.wer(bench.norm(it['spoken']),bench.norm(text))))
        group=[r for r in rows if r['language']==lang and r['beam']==beam]
        print(lang,beam,'WER',round(sum(r['wer'] for r in group)/len(group),3),'mean ms',round(sum(r['ms'] for r in group)/len(group)),flush=True)
finally:
    p.terminate();p.wait(timeout=15)
with open('eval/optimization-results.json','w',encoding='utf-8') as f: json.dump(rows,f,ensure_ascii=False,indent=2)
