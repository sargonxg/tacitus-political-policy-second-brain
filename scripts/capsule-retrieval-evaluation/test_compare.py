import copy,json,pathlib,unittest,os
from compare import baseline,coverage,encoded
from scale import fixture
ROOT=pathlib.Path(os.environ['TACITUS_FIXTURES'])
class Tests(unittest.TestCase):
 def setUp(self):
  self.c=json.loads((ROOT/'knowledge/capture.json').read_text());self.q={'question':'heat exercise','known_as_of':300,'valid_at':300,'time_mode':'disclose_unknown','max_bytes':8192,'revoked_sources':[]}
 def test_chunk_is_not_whole_document_handicap(self):
  self.assertEqual(coverage(self.c,baseline(self.c,self.q,True),'line_chunk'),['heat'])
  self.assertIn('injection',coverage(self.c,baseline(self.c,self.q),'whole_document'))
 def test_exact_repeat_and_budget(self):
  for budget in [2048,8192,16384,32768]:
   self.q['max_bytes']=budget
   for chunk in [False,True]:
    b=baseline(self.c,self.q,chunk);self.assertLessEqual(len(b),budget);self.assertEqual(b,baseline(self.c,self.q,chunk))
 def test_temporal_and_revoked_refusal(self):
  for key,value in [('known_as_of',299),('time_mode','require_declared_interval'),('revoked_sources',['unknown'])]:
   q=dict(self.q);q[key]=value
   with self.assertRaises(ValueError):baseline(self.c,q,True)
 def test_same_text_restriction_refusal(self):
  for field,value in [('withdrawn',True),('restricted',True),('available_at',301),('valid_until',300)]:
   c=copy.deepcopy(self.c);c['documents'][0][field]=value
   with self.assertRaises(ValueError):baseline(c,self.q)
 def test_synthetic_shards_have_unique_revision_bound_records(self):
  c=fixture(self.c,range(25));self.assertEqual(len(c['documents']),25);self.assertEqual(len(c['assertions']),50)
  self.assertEqual(len({d['id']for d in c['documents']}),25)
 def test_french_regression_baseline_retrieves_original(self):
  c=json.loads((ROOT/'minister/capture.json').read_text());q=dict(self.q);q.update(question='applicable',known_as_of=c['manifest']['created_at'],valid_at=c['manifest']['created_at'])
  self.assertIn('application-fr',coverage(c,baseline(c,q,True),'line_chunk'))
 def test_corrupt_same_length_text_or_chunk_is_not_coverage(self):
  for chunk in [False,True]:
   value=json.loads(baseline(self.c,self.q,chunk))
   if chunk:value['chunks'][0]['quote']='X'*len(value['chunks'][0]['quote'])
   else:value['documents'][0]['text']='X'*len(value['documents'][0]['text'])
   self.assertNotIn('heat',coverage(self.c,encoded(value),'line_chunk' if chunk else 'whole_document'))
 def test_forged_capsule_citation_is_not_coverage(self):
  claim=self.c['assertions'][1];value={'context':{'claims':{'heat':{'assertion':claim,'citations':[{'span':claim['evidence'][0],'text':'invented'}]}}}}
  self.assertEqual(coverage(self.c,encoded(value),'capsule'),[])
if __name__=='__main__':unittest.main()
