-- Fix permission name typo: VIEWRE_PORTS -> VIEW_REPORTS
UPDATE permissions SET name = 'VIEW_REPORTS' WHERE name = 'VIEWRE_PORTS';
